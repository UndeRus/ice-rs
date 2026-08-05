//! Аутентификатор на `HashMap`: два пользователя, одного переименовываем и
//! кладём в группу.
//!
//! ```sh
//! cargo run -p mumble-ice --example authenticator
//! ```
//!
//! Проверить, не подключая клиента, можно через `Server::verifyPassword` — Murmur
//! отправляет его прямо в `authenticate`:
//!
//! ```sh
//! # в другом терминале, через тот же фасад
//! cargo run -p mumble-ice --example roster
//! ```

use async_trait::async_trait;
use mumble_ice::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

struct Account {
    id: UserId,
    password: String,
    rename_to: Option<String>,
    groups: Vec<String>,
}

struct MapAuth {
    by_name: HashMap<String, Account>,
}

impl MapAuth {
    fn demo() -> MapAuth {
        let mut by_name = HashMap::new();
        by_name.insert(
            String::from("alice"),
            Account {
                id: UserId(1001),
                password: String::from("alice-pw"),
                // Переименование действует на время сессии.
                rename_to: Some(String::from("Alice [staff]")),
                // Группа корневого канала — тоже на время сессии.
                groups: vec![String::from("admin")],
            },
        );
        by_name.insert(
            String::from("bob"),
            Account {
                id: UserId(1002),
                password: String::from("bob-pw"),
                rename_to: None,
                groups: vec![],
            },
        );
        MapAuth { by_name }
    }

    fn by_id(&self, id: UserId) -> Option<(&str, &Account)> {
        self.by_name
            .iter()
            .find(|(_, a)| a.id == id)
            .map(|(n, a)| (n.as_str(), a))
    }
}

#[async_trait]
impl Authenticator for MapAuth {
    async fn authenticate(&self, req: AuthRequest) -> AuthResult {
        // Murmur сравнивает имена без учёта регистра — делаем так же.
        let acct = match self.by_name.get(&req.name_ci()) {
            Some(a) => a,
            // ВАЖНО: незнакомое имя — это FallThrough, а не Denied.
            // С Denied все пользователи из базы Murmur'а перестанут входить.
            None => return AuthResult::FallThrough,
        };

        if acct.password != req.password {
            return AuthResult::Denied;
        }

        let mut ok = AuthOk::new(acct.id);
        if let Some(new) = &acct.rename_to {
            ok = ok.rename(new);
        }
        AuthResult::Ok(ok.groups(acct.groups.clone()))
    }

    /// Чтобы редактор ACL и лог показывали имена, а не числа.
    async fn name_to_id(&self, name: &str) -> Lookup<UserId> {
        Lookup::from_option(self.by_name.get(&name.to_lowercase()).map(|a| a.id))
    }

    async fn id_to_name(&self, id: UserId) -> Lookup<String> {
        Lookup::from_option(self.by_id(id).map(|(n, _)| n.to_string()))
    }

    async fn user_info(&self, id: UserId) -> Lookup<UserInfo> {
        match self.by_id(id) {
            Some((name, _)) => Lookup::Found(
                UserInfo::new(name).with_email(format!("{}@example.invalid", name)),
            ),
            None => Lookup::Unknown,
        }
    }

    /// Иначе наши пользователи не появятся в `Server::getRegisteredUsers`.
    async fn registered_users(&self, filter: &str) -> Lookup<BTreeMap<UserId, String>> {
        let f = filter.to_lowercase();
        Lookup::Found(
            self.by_name
                .iter()
                .filter(|(n, _)| f.is_empty() || n.contains(&f))
                .map(|(n, a)| (a.id, n.clone()))
                .collect(),
        )
    }

    async fn on_error(&self, err: mumble_ice::Error) {
        eprintln!("аутентификатор: {}", err);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> mumble_ice::Result<()> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:6502"));

    let mut builder = MurmurClient::builder()
        .endpoint(endpoint.as_str())?
        .callback_listen("127.0.0.1:0".parse().unwrap());
    if let Ok(secret) = std::env::var("MUMBLE_ICE_SECRET") {
        builder = builder.secret(secret);
    }
    let client = builder.connect().await?;
    let srv = client.only_server().await?;

    let sub = srv.set_authenticator(Arc::new(MapAuth::demo())).await?;
    println!("аутентификатор установлен на сервере {}", srv.id());

    // Проверка, не поднимая Mumble-клиента: verifyPassword идёт в authenticate.
    println!(
        "alice с верным паролем: {:?}",
        srv.verify_password("alice", "alice-pw").await?
    );
    println!(
        "alice с неверным паролем: {:?}",
        srv.verify_password_detailed("alice", "нет").await?
    );
    println!(
        "незнакомое имя (FallThrough → база Murmur'а): {:?}",
        srv.verify_password_detailed("charlie", "x").await?
    );

    println!("Ctrl-C, чтобы снять аутентификатор и выйти");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        err = sub.closed() => eprintln!("подписка умерла: {}", err),
    }

    // Снятие возвращает аутентификацию базе Murmur'а.
    client.shutdown().await;
    Ok(())
}

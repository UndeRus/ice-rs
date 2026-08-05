//! Бот: приветствует входящих и понимает пару команд в чате.
//!
//! ```sh
//! cargo run -p mumble-ice --example bot
//! ```
//!
//! Секрет — через `MUMBLE_ICE_SECRET`. Под Docker/NAT добавьте
//! `.callback_advertise(host, port)`: Murmur звонит на этот адрес **наружу**.

use async_trait::async_trait;
use mumble_ice::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Bot {
    greeted: Mutex<HashSet<String>>,
}

#[async_trait]
impl ServerEvents for Bot {
    async fn user_connected(&self, srv: &VirtualServer, user: User) -> mumble_ice::Result<()> {
        // Состояние бота — одна структура рядом с обработчиками, без Arc на
        // каждое замыкание.
        if self.greeted.lock().await.insert(user.name.clone()) {
            srv.message_user(user.session, &format!("Привет, {}!", user.name))
                .await?;
        }
        Ok(())
    }

    async fn user_text_message(
        &self,
        srv: &VirtualServer,
        user: User,
        msg: TextMessage,
    ) -> mumble_ice::Result<()> {
        match msg.text.trim() {
            "!кто" => {
                let names: Vec<String> = srv.users().await?.into_iter().map(|u| u.name).collect();
                srv.message_user(user.session, &format!("Онлайн: {}", names.join(", ")))
                    .await?;
            }
            "!afk" => {
                match srv.find_channel_by_name("AFK").await? {
                    Some(ch) => srv.move_user(user.session, ch.id).await?,
                    None => {
                        srv.message_user(user.session, "Канала AFK нет")
                            .await?
                    }
                }
            }
            "!права" => {
                let p = srv.effective_permissions(user.session, user.channel).await?;
                srv.message_user(
                    user.session,
                    &format!(
                        "Здесь у вас: speak={} write={} kick={}",
                        p.contains(Permission::SPEAK),
                        p.contains(Permission::WRITE),
                        p.contains(Permission::KICK)
                    ),
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn reattached(&self, srv: &VirtualServer) -> mumble_ice::Result<()> {
        // Виртуальный сервер перезапустился: все прежние сессии умерли, значит и
        // «кого уже приветствовали» больше не актуально.
        self.greeted.lock().await.clear();
        srv.broadcast("бот вернулся").await
    }

    async fn on_error(&self, err: mumble_ice::Error) {
        // Ошибка обработчика не снимает подписку, но и не должна теряться.
        eprintln!("бот: {}", err);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> mumble_ice::Result<()> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:6502"));

    let mut builder = MurmurClient::builder()
        .endpoint(endpoint.as_str())?
        // Адрес прослушивания для входящих вызовов Murmur'а.
        .callback_listen("127.0.0.1:0".parse().unwrap());
    if let Ok(secret) = std::env::var("MUMBLE_ICE_SECRET") {
        builder = builder.secret(secret);
    }
    let client = builder.connect().await?;
    let srv = client.only_server().await?;

    let sub = srv
        .on_events(Arc::new(Bot {
            greeted: Mutex::new(HashSet::new()),
        }))
        .await?;

    println!("бот работает на сервере {}; Ctrl-C для выхода", srv.id());

    // `closed()` разрешается, если подписка умерла безвозвратно — чтобы бот не
    // сидел глухим.
    tokio::select! {
        _ = tokio::signal::ctrl_c() => println!("выходим"),
        err = sub.closed() => eprintln!("подписка умерла: {}", err),
    }

    client.shutdown().await;
    Ok(())
}

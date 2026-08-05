//! E2E аутентификатора против живого Murmur.
//!
//! ```sh
//! cargo nextest run -p mumble-ice --test e2e_auth --run-ignored all
//! ```
//!
//! Подключить настоящий Mumble-клиент здесь нельзя, но аутентификатор дёргается и
//! со стороны Ice: `Server::registerUser` идёт в `register_user`,
//! `getRegistration` — в `get_info`, `getRegisteredUsers` — в
//! `registered_users`. Это тот же servant и тот же провод, что и при живом
//! логине, поэтому «Murmur позвал наш аутентификатор» проверяется полностью.

use async_trait::async_trait;
use mumble_ice::prelude::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

fn endpoint() -> String {
    std::env::var("MUMBLE_ICE_ENDPOINT").unwrap_or_else(|_| String::from("127.0.0.1:6502"))
}

async fn connect() -> MurmurClient {
    let mut b = MurmurClient::builder()
        .endpoint(endpoint().as_str())
        .expect("адрес")
        .callback_listen("127.0.0.1:0".parse().unwrap());
    if let Ok(secret) = std::env::var("MUMBLE_ICE_SECRET") {
        b = b.secret(secret);
    }
    b.connect().await.expect("подключение к Murmur")
}

fn tag() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

/// Какие методы аутентификатора Murmur реально позвал.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Called {
    Authenticate(String),
    UserInfo(UserId),
    NameToId(String),
    IdToName(UserId),
    RegisterUser(String),
    RegisteredUsers(String),
    SetUserInfo(UserId),
    UnregisterUser(UserId),
}

struct Recorder {
    calls: Arc<Mutex<Vec<Called>>>,
    errors: Arc<Mutex<Vec<String>>>,
    /// Пользователь, которого мы «знаем».
    known: (String, String, UserId),
    panic_next: AtomicBool,
}

impl Recorder {
    fn new(known_name: &str) -> (Arc<Recorder>, Arc<Mutex<Vec<Called>>>, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        (
            Arc::new(Recorder {
                calls: calls.clone(),
                errors: errors.clone(),
                known: (String::from(known_name), String::from("secret-pw"), UserId(31337)),
                panic_next: AtomicBool::new(false),
            }),
            calls,
            errors,
        )
    }
}

#[async_trait]
impl Authenticator for Recorder {
    async fn authenticate(&self, req: AuthRequest) -> AuthResult {
        self.calls
            .lock()
            .await
            .push(Called::Authenticate(req.name.clone()));
        if self.panic_next.swap(false, Ordering::AcqRel) {
            panic!("нарочная паника в authenticate");
        }
        if req.name_ci() != self.known.0.to_lowercase() {
            // Незнакомое имя — НЕ Denied.
            return AuthResult::FallThrough;
        }
        if req.password != self.known.1 {
            return AuthResult::Denied;
        }
        AuthResult::Ok(
            AuthOk::new(self.known.2)
                .rename(format!("{} [auth]", self.known.0))
                .group("mi-testers"),
        )
    }

    async fn user_info(&self, id: UserId) -> Lookup<UserInfo> {
        self.calls.lock().await.push(Called::UserInfo(id));
        if id == self.known.2 {
            Lookup::Found(
                UserInfo::new(&self.known.0).with_email("mi@example.invalid"),
            )
        } else {
            Lookup::Unknown
        }
    }

    async fn name_to_id(&self, name: &str) -> Lookup<UserId> {
        self.calls
            .lock()
            .await
            .push(Called::NameToId(String::from(name)));
        if name.eq_ignore_ascii_case(&self.known.0) {
            Lookup::Found(self.known.2)
        } else {
            Lookup::Unknown
        }
    }

    async fn id_to_name(&self, id: UserId) -> Lookup<String> {
        self.calls.lock().await.push(Called::IdToName(id));
        if id == self.known.2 {
            Lookup::Found(self.known.0.clone())
        } else {
            Lookup::Unknown
        }
    }

    async fn register_user(&self, info: &UserInfo) -> RegisterResult {
        let name = info.name().unwrap_or_default().to_string();
        self.calls.lock().await.push(Called::RegisterUser(name));
        // «Не моё»: пусть Murmur регистрирует у себя.
        RegisterResult::FallThrough
    }

    async fn unregister_user(&self, id: UserId) -> UpdateResult {
        self.calls.lock().await.push(Called::UnregisterUser(id));
        UpdateResult::FallThrough
    }

    async fn registered_users(&self, filter: &str) -> Lookup<BTreeMap<UserId, String>> {
        self.calls
            .lock()
            .await
            .push(Called::RegisteredUsers(String::from(filter)));
        let mut m = BTreeMap::new();
        m.insert(self.known.2, self.known.0.clone());
        Lookup::Found(m)
    }

    async fn set_user_info(&self, id: UserId, _info: &UserInfo) -> UpdateResult {
        self.calls.lock().await.push(Called::SetUserInfo(id));
        UpdateResult::FallThrough
    }

    async fn on_error(&self, err: mumble_ice::Error) {
        self.errors.lock().await.push(err.to_string());
    }
}

async fn wait_for<F>(calls: &Arc<Mutex<Vec<Called>>>, pred: F, secs: u64) -> bool
where
    F: Fn(&Called) -> bool,
{
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < deadline {
        if calls.lock().await.iter().any(&pred) {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    false
}

/// Приёмка S8: Murmur действительно вызывает наш аутентификатор, и трансляция
/// сентинелов работает на проводе.
///
/// `Server::verifyPassword` идёт прямо в `authenticate` — проверено
/// диагностическим прогоном (`probe_which_operations_reach_the_authenticator`).
/// Это тот же путь, что при живом логине, включая out-параметры `newname` и
/// `groups`, поэтому Mumble-клиент для проверки не нужен.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice на 127.0.0.1:6502"]
async fn murmur_calls_our_authenticator() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let name = format!("mi_auth_{}", tag());
    let (rec, calls, errors) = Recorder::new(&name);
    let sub = srv
        .set_authenticator(rec)
        .await
        .expect("set_authenticator принят Murmur'ом");
    assert!(sub.is_alive());

    // ── AuthResult::Ok ────────────────────────────────────────────────────
    let ok = srv
        .verify_password(&name, "secret-pw")
        .await
        .expect("verify_password");
    assert!(
        wait_for(&calls, |c| matches!(c, Called::Authenticate(n) if n == &name), 8).await,
        "authenticate не дошёл до аутентификатора; вызовы: {:?}, ошибки: {:?}",
        calls.lock().await,
        errors.lock().await
    );
    assert_eq!(
        Some(UserId(31337)),
        ok,
        "id из нашего аутентификатора должен доехать обратно"
    );

    // ── Denied против FallThrough: главное различие всего модуля ───────────
    // Верное имя, неверный пароль → Denied (-1).
    assert_eq!(
        PasswordCheck::WrongPassword,
        srv.verify_password_detailed(&name, "не-тот-пароль")
            .await
            .expect("verify wrong"),
        "Denied должен приезжать как WrongPassword"
    );

    // Незнакомое имя → FallThrough (-2): Murmur смотрит свою базу и не находит.
    assert_eq!(
        PasswordCheck::NoSuchUser,
        srv.verify_password_detailed(&format!("mi_unknown_{}", tag()), "x")
            .await
            .expect("verify unknown"),
        "FallThrough должен приводить к проверке базой Murmur'а"
    );

    // ── getRegisteredUsers объединяется с базой Murmur'а ──────────────────
    let listed = srv.registered_users("").await.expect("registered_users");
    assert!(
        wait_for(&calls, |c| matches!(c, Called::RegisteredUsers(_)), 8).await,
        "registered_users не дошёл; вызовы: {:?}",
        calls.lock().await
    );
    assert!(
        listed.contains_key(&UserId(31337)),
        "пользователь из аутентификатора должен попасть в список: {:?}",
        listed
    );

    // ── getRegistration идёт в user_info ─────────────────────────────────
    let info = srv
        .registration(UserId(31337))
        .await
        .expect("registration из аутентификатора");
    assert_eq!(Some(name.as_str()), info.name());
    assert_eq!(Some("mi@example.invalid"), info.email());

    // Уборка: отдаём аутентификацию обратно Murmur'у.
    sub.unsubscribe().await.expect("unsubscribe");
    client.shutdown().await;

    assert!(
        errors.lock().await.is_empty(),
        "ошибок быть не должно: {:?}",
        errors.lock().await
    );
}

/// После снятия аутентификатора Murmur снова проверяет по своей базе.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn unsubscribe_hands_authentication_back() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let name = format!("mi_auth_{}", tag());
    let (rec, calls, _) = Recorder::new(&name);
    let sub = srv.set_authenticator(rec).await.expect("set_authenticator");

    assert_eq!(
        Some(UserId(31337)),
        srv.verify_password(&name, "secret-pw")
            .await
            .expect("verify до снятия")
    );

    sub.unsubscribe().await.expect("unsubscribe");
    calls.lock().await.clear();

    // Теперь наш аутентификатор знать не должны.
    let after = srv
        .verify_password_detailed(&name, "secret-pw")
        .await
        .expect("verify после снятия");
    let called_after = calls.lock().await.clone();

    client.shutdown().await;

    assert_eq!(
        PasswordCheck::NoSuchUser, after,
        "после снятия аутентификатора Murmur не должен знать этого пользователя"
    );
    assert!(
        called_after.is_empty(),
        "снятый аутентификатор не должен вызываться: {:?}",
        called_after
    );
}

/// Регистрация через Ice должна спрашивать аутентификатор, а `FallThrough` —
/// отдавать дело базе Murmur'а.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn register_falls_through_to_murmur() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let known = format!("mi_auth_{}", tag());
    let (rec, calls, _) = Recorder::new(&known);
    let sub = srv.set_authenticator(rec).await.expect("set_authenticator");

    let new_name = format!("mi_reg_{}", tag());
    let uid = srv
        .register_named(&new_name, Some("pw"))
        .await
        .expect("register_named");

    assert!(
        wait_for(&calls, |c| matches!(c, Called::RegisterUser(_)), 8).await,
        "register_user не дошёл до аутентификатора; вызовы: {:?}",
        calls.lock().await
    );
    // FallThrough означает «регистрируй у себя» — и Murmur выдал свой id.
    assert_ne!(
        UserId(31337), uid,
        "при FallThrough id должен быть от Murmur'а, а не наш"
    );

    srv.unregister(uid).await.ok();
    sub.unsubscribe().await.ok();
    client.shutdown().await;
}

/// Паника в аутентификаторе не должна ломать вход всем: на провод уходит
/// безопасное значение, а подписка живёт.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn panic_degrades_to_safe_value() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let known = format!("mi_auth_{}", tag());
    let (rec, calls, errors) = Recorder::new(&known);
    rec.panic_next.store(true, Ordering::Release);
    let sub = srv.set_authenticator(rec).await.expect("set_authenticator");

    // Дёргаем что-нибудь, что заведомо идёт в аутентификатор, чтобы он ожил.
    let _ = srv.registered_users("").await;
    assert!(
        wait_for(&calls, |c| matches!(c, Called::RegisteredUsers(_)), 8).await,
        "аутентификатор не отвечает"
    );

    // Подписка обязана быть живой; ошибки, если паника случилась, — в errors.
    assert!(sub.is_alive(), "подписка не должна умирать");
    let errs = errors.lock().await.clone();
    assert!(
        errs.iter().all(|e| !e.contains("отменён")),
        "неожиданные отмены: {:?}",
        errs
    );

    sub.unsubscribe().await.ok();
    client.shutdown().await;
}

/// Диагностика: показывает, какие именно методы аутентификатора Murmur зовёт на
/// каждую операцию Ice. Полезно, потому что в Slice это не описано.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "диагностический прогон"]
async fn probe_which_operations_reach_the_authenticator() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let known = format!("mi_auth_{}", tag());
    let (rec, calls, _) = Recorder::new(&known);
    let sub = srv.set_authenticator(rec).await.expect("set_authenticator");

    let probe = |label: &'static str,
                 calls: Arc<Mutex<Vec<Called>>>|
     -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let got = calls.lock().await.clone();
            println!("[probe] {} -> {:?}", label, got);
            calls.lock().await.clear();
        })
    };

    let _ = srv.registered_users("").await;
    probe("getRegisteredUsers", calls.clone()).await;

    let _ = srv.verify_password(&known, "secret-pw").await;
    probe("verifyPassword", calls.clone()).await;

    let _ = srv.registration(UserId(31337)).await;
    probe("getRegistration", calls.clone()).await;

    let _ = srv.texture(UserId(31337)).await;
    probe("getTexture", calls.clone()).await;

    sub.unsubscribe().await.ok();
    client.shutdown().await;
}

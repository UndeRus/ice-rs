//! E2E против живого Murmur.
//!
//! ```sh
//! cargo nextest run -p mumble-ice --run-ignored all
//! ```
//!
//! Требуется Murmur с Ice на `127.0.0.1:6502` и хотя бы одним запущенным
//! виртуальным сервером. Переменные: `MUMBLE_ICE_ENDPOINT`, `MUMBLE_ICE_SECRET`.
//!
//! Тесты обязаны быть на multi_thread-рантайме: `Proxy::from_bytes` в нижнем
//! слое дозванивается по TCP через `block_on` внутри десериализации, и на
//! current_thread это дедлок.

use mumble_ice::prelude::*;

fn endpoint() -> String {
    std::env::var("MUMBLE_ICE_ENDPOINT").unwrap_or_else(|_| String::from("127.0.0.1:6502"))
}

async fn connect() -> MurmurClient {
    let mut b = MurmurClient::builder()
        .endpoint(endpoint().as_str())
        .expect("адрес");
    if let Ok(secret) = std::env::var("MUMBLE_ICE_SECRET") {
        b = b.secret(secret);
    }
    b.connect().await.expect("подключение к Murmur")
}

/// Уникальный суффикс, чтобы параллельные прогоны не мешали друг другу.
fn tag() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice на 127.0.0.1:6502"]
async fn meta_level_reads() {
    let client = connect().await;

    let v = client.version().await.expect("version");
    assert!(v.major > 0, "версия выглядит пустой: {:?}", v);
    // Заменяет четыре &mut out-параметра.
    assert!(v.to_string().contains(&v.major.to_string()), "{}", v);

    assert!(client.uptime().await.expect("uptime").as_secs() < 60 * 60 * 24 * 365);
    assert!(!client.slice_source().await.expect("slice").is_empty());
    assert!(!client
        .slice_checksums()
        .await
        .expect("checksums")
        .is_empty());
    assert!(!client
        .default_config()
        .await
        .expect("default_config")
        .is_empty());

    // DbState вместо Dbstate. Murmur 1.5.857 эту операцию не реализует, хотя она
    // есть в Slice, — поэтому отдельный вариант ошибки, а не паника.
    match client.database_state().await {
        Ok(st) => client
            .set_database_state(st)
            .await
            .expect("set_database_state"),
        Err(Error::OperationNotSupported { operation }) => {
            eprintln!("сервер не реализует {} — пропускаем", operation)
        }
        Err(other) => panic!("database_state: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn discovers_servers_and_caches_the_handle() {
    let client = connect().await;

    let booted = client.booted_servers().await.expect("booted_servers");
    assert!(!booted.is_empty(), "нужен запущенный виртуальный сервер");
    let id = booted[0].id();
    assert!(id.get() > 0, "id должен быть разрешён: {:?}", id);

    // only_server — сахар для типичного бота.
    let srv = client.only_server().await.expect("only_server");
    assert!(srv.is_running().await.expect("is_running"));

    // Повторный server() отдаёт закэшированный хендл.
    let a = client.server(id).await.expect("server");
    let b = client.server(id).await.expect("server дважды");
    assert_eq!(a.id(), b.id());

    assert!(!client.servers().await.expect("servers").is_empty());
}

/// Ключевое отличие фасада: хендл `Clone`, методы `&self`, поэтому его можно
/// раздать по таскам без `Arc<Mutex<_>>` в пользовательском коде.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn handle_is_shareable_across_tasks() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let mut tasks = Vec::new();
    for _ in 0..4 {
        let srv = srv.clone();
        tasks.push(tokio::spawn(async move {
            srv.channels().await.map(|c| c.len())
        }));
    }
    for t in tasks {
        let n = t.await.expect("таск").expect("channels");
        assert!(n >= 1, "как минимум корневой канал");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn channel_lifecycle_with_typed_ids() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let root = srv.channel(ChannelId::ROOT).await.expect("root channel");
    assert!(root.is_root());
    assert_eq!(None, root.parent, "у корня нет родителя");

    let name = format!("mi_chan_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");

    let found = srv
        .find_channel_by_name(&name)
        .await
        .expect("find_channel_by_name");
    assert_eq!(Some(id), found.map(|c| c.id));

    // read-modify-write вместо сборки структуры с нуля.
    srv.set_channel_description(id, "описание от mumble-ice")
        .await
        .expect("set_channel_description");
    let ch = srv.channel(id).await.expect("channel");
    assert_eq!("описание от mumble-ice", ch.description);
    assert_eq!(Some(ChannelId::ROOT), ch.parent);

    srv.rename_channel(id, &format!("{}_renamed", name))
        .await
        .expect("rename_channel");
    assert!(srv
        .channel(id)
        .await
        .expect("channel")
        .name
        .ends_with("_renamed"));

    // Дерево — настоящий рекурсивный тип, а не Vec<Box<Tree>>.
    let tree = srv.tree().await.expect("tree");
    assert!(tree.channel.is_root());
    assert!(
        tree.find(id).is_some(),
        "созданный канал должен найтись в дереве"
    );
    assert!(tree.walk().len() >= 2);

    srv.remove_channel(id).await.expect("remove_channel");

    // Отсутствующий канал: try_* даёт None, а не ошибку.
    assert_eq!(None, srv.try_channel(id).await.expect("try_channel"));
    // А строгий вызов — ошибку, по которой можно ветвиться.
    let err = srv.channel(id).await.expect_err("канала больше нет");
    assert!(err.is_stale_handle(), "{:?}", err);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn registered_user_lifecycle() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let name = format!("mi_user_{}", tag());
    let uid = srv
        .register_named(&name, Some("pw-12345"))
        .await
        .expect("register_named");

    let info = srv.registration(uid).await.expect("registration");
    assert_eq!(Some(name.as_str()), info.name());
    // Пароль не должен попадать в Debug.
    assert!(!format!("{:?}", info).contains("pw-12345"));

    let all = srv.registered_users("").await.expect("registered_users");
    assert!(all.contains_key(&uid), "новый пользователь в списке");

    // Сентинелы verifyPassword спрятаны в enum.
    assert_eq!(
        Some(uid),
        srv.verify_password(&name, "pw-12345")
            .await
            .expect("verify_password")
    );
    assert_eq!(
        PasswordCheck::WrongPassword,
        srv.verify_password_detailed(&name, "wrong")
            .await
            .expect("verify wrong")
    );
    assert_eq!(
        PasswordCheck::NoSuchUser,
        srv.verify_password_detailed("mi_nobody_zzz", "x")
            .await
            .expect("verify unknown")
    );

    // Аватар: пустой ответ — это None, а не пустой Vec.
    assert_eq!(None, srv.texture(uid).await.expect("texture"));

    srv.update_registration(uid, info.with_comment("комментарий"))
        .await
        .expect("update_registration");
    assert_eq!(
        Some("комментарий"),
        srv.registration(uid)
            .await
            .expect("registration")
            .comment()
    );

    srv.unregister(uid).await.expect("unregister");
    let err = srv.registration(uid).await.expect_err("больше не зарегистрирован");
    assert!(err.is_stale_handle(), "{:?}", err);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn acl_round_trip_with_bitflags() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let snap = srv.acl(ChannelId::ROOT).await.expect("acl");
    let own_before = snap.own_acls().count();

    // Права — bitflags, а не 0x01 руками.
    let group = format!("mi_grp_{}", tag());
    srv.update_acl(ChannelId::ROOT, |s| {
        s.allow(
            AclSubject::Group(group.clone()),
            Permission::ENTER | Permission::SPEAK,
        );
    })
    .await
    .expect("update_acl");

    let after = srv.acl(ChannelId::ROOT).await.expect("acl снова");
    assert_eq!(
        own_before + 1,
        after.own_acls().count(),
        "запись должна добавиться"
    );
    let added = after
        .own_acls()
        .find(|a| a.subject == AclSubject::Group(group.clone()))
        .expect("наша запись");
    assert!(added.allow.contains(Permission::ENTER));
    assert!(added.allow.contains(Permission::SPEAK));
    assert!(!added.allow.contains(Permission::KICK));

    // Убираем за собой.
    srv.update_acl(ChannelId::ROOT, |s| {
        s.acls
            .retain(|a| a.subject != AclSubject::Group(group.clone()));
    })
    .await
    .expect("откат acl");
    assert_eq!(own_before, srv.acl(ChannelId::ROOT).await.unwrap().own_acls().count());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn config_log_and_bans() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let all = srv.all_config().await.expect("all_config");
    assert!(!all.is_empty());

    // Пустое значение — None, а не "".
    assert_eq!(
        None,
        srv.config("mi_definitely_absent_key")
            .await
            .expect("config")
    );

    // Лог пополняется от наших же вызовов, поэтому точное число сравнивать
    // нельзя. Зато можно пришпилить семантику границ: getLog — полуинтервал.
    let len = srv.log_len().await.expect("log_len");
    assert!(len > 0, "лог не должен быть пуст после наших вызовов");
    assert_eq!(
        1,
        srv.log(0, 1).await.expect("log(0,1)").len(),
        "getLog(first, end) — полуинтервал: [0,1) это ровно одна запись"
    );
    assert!(srv.log(0, 0).await.expect("log(0,0)").is_empty());
    assert!(!srv.all_log().await.expect("all_log").is_empty());

    // Баны: пустой список туда-обратно не должен ничего сломать.
    let bans = srv.bans().await.expect("bans");
    srv.replace_bans(&bans).await.expect("replace_bans");
    assert_eq!(bans.len(), srv.bans().await.expect("bans снова").len());
}

/// Escape hatch должен работать и нести готовый контекст.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn raw_escape_hatch() {
    use murmur_slice::mumble_server::Server as _;

    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let mut raw = srv.raw().await;
    let ctx = raw.ctx();
    let conf = raw.get_all_conf(ctx).await.expect("get_all_conf через raw");
    assert!(!conf.is_empty());
}

/// Неверный секрет должен приезжать конкретным вариантом, а не строкой.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice И непустой icesecretread"]
async fn wrong_secret_maps_to_invalid_secret() {
    let client = MurmurClient::builder()
        .endpoint(endpoint().as_str())
        .expect("адрес")
        .secret("определённо-неверный-секрет")
        .connect()
        .await
        .expect("подключение");

    match client.uptime().await {
        Err(Error::InvalidSecret) => {}
        Err(other) => panic!("ожидали InvalidSecret, получили {:?}", other),
        Ok(_) => eprintln!("пропущено: у сервера пустой icesecretread"),
    }
}

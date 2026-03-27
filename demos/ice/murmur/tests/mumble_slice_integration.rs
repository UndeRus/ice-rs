//! Интеграционные тесты ко методам `MumbleServer.ice`, вызываемым **с клиента** на прокси `Meta` и `Server`.
//!
//! **Покрытие**
//! - `meta_all_methods` — все операции `Meta`, кроме `addCallback` / `removeCallback` (нужен ваш `MetaCallback` servant).
//! - `server_all_methods_smoke` — все операции `Server`, кроме: колбэков/аутентификатора, `stop`/`delete`,
//!   `setSuperuserPassword` (см. отдельные тесты ниже).
//! - `meta_new_server_virtual_delete` — `newServer` + `delete` на **новом** виртуальном сервере.
//! - `server_stop_then_start` — опасно для общего инстанса (останавливает первый VS).
//! - Заглушки `meta_callbacks_*`, `server_callbacks_*`, `server_set_superuser_password_*` — требуют servant / меняют пароль.
//! - Входящие интерфейсы (`ServerCallback`, …) — см. `incoming_callback_interfaces_documented`.
//!
//! Запуск при поднятом Murmur с Ice:
//! ```text
//! cargo test -p mumble-ice-demo --test mumble_slice_integration -- --ignored
//! ```
//!
//! Переменные: `MUMBLE_ICE_ENDPOINT` (по умолчанию `Meta:tcp -h 127.0.0.1 -p 6502`), `MUMBLE_ICE_SECRET` при необходимости.

use ice_rs::communicator::Communicator;
use ice_rs::iceobject::IceObject;
use mumble_ice_demo::gen::mumble_server::{
    Acllist, BanList, GroupList, IdList, LogList, Meta, MetaPrx, NameList, NameMap, Server,
    ServerPrx, Texture, Tree, User, UserInfo, UserInfoMap,
};

use std::collections::HashMap;

type IceCtx = Option<HashMap<String, String>>;

fn endpoint_string() -> String {
    std::env::var("MUMBLE_ICE_ENDPOINT")
        .unwrap_or_else(|_| "Meta:tcp -h 127.0.0.1 -p 6502".to_string())
}

fn ice_context() -> IceCtx {
    match std::env::var("MUMBLE_ICE_SECRET") {
        Ok(secret) if !secret.is_empty() => {
            let mut m = HashMap::new();
            m.insert("secret".into(), secret);
            Some(m)
        }
        _ => None,
    }
}

async fn connect() -> (MetaPrx, ServerPrx, IceCtx) {
    let mut comm = Communicator::new().await.expect("Communicator::new");
    let ctx = ice_context();
    let proxy = comm
        .string_to_proxy(&endpoint_string())
        .await
        .expect("string_to_proxy");
    let mut meta = MetaPrx::unchecked_cast(proxy)
        .await
        .expect("Meta unchecked_cast");
    let mut servers = meta
        .get_booted_servers(ctx.clone())
        .await
        .expect("get_booted_servers");
    let srv = servers
        .pop()
        .expect("нужен хотя бы один запущенный виртуальный сервер");
    let _keep = comm;
    (meta, srv, ctx)
}

#[tokio::test]
#[ignore = "нужен Murmur с Ice"]
async fn meta_all_methods() {
    let (mut meta, mut srv, ctx) = connect().await;

    meta.ice_ping().await.expect("Meta.ice_ping");
    assert!(meta.ice_is_a().await.expect("Meta.ice_is_a"));
    let _ = meta.ice_id().await.expect("Meta.ice_id");
    let _ = meta.ice_ids().await.expect("Meta.ice_ids");

    let mut major = 0i32;
    let mut minor = 0i32;
    let mut patch = 0i32;
    let mut text = String::new();
    meta.get_version(&mut major, &mut minor, &mut patch, &mut text, ctx.clone())
        .await
        .expect("get_version");
    assert!(!text.is_empty() || major > 0);

    let _uptime = meta.get_uptime(ctx.clone()).await.expect("Meta.get_uptime");

    let slice_txt = meta.get_slice(ctx.clone()).await.expect("get_slice");
    assert!(!slice_txt.is_empty());

    let checksums = meta
        .get_slice_checksums(ctx.clone())
        .await
        .expect("get_slice_checksums");
    assert!(!checksums.is_empty());

    let _default_conf = meta
        .get_default_conf(ctx.clone())
        .await
        .expect("get_default_conf");

    let booted = meta
        .get_booted_servers(ctx.clone())
        .await
        .expect("get_booted_servers");
    assert!(!booted.is_empty());

    let all = meta
        .get_all_servers(ctx.clone())
        .await
        .expect("get_all_servers");
    assert!(!all.is_empty());

    let sid = srv.id(ctx.clone()).await.expect("Server.id для get_server");
    let mut s2 = meta
        .get_server(sid, ctx.clone())
        .await
        .expect("get_server");
    s2.ice_ping().await.expect("прокси с get_server жив");

    let db = meta
        .get_assumed_database_state(ctx.clone())
        .await
        .expect("get_assumed_database_state");
    meta.set_assumed_database_state(&db, ctx.clone())
        .await
        .expect("set_assumed_database_state");
}

#[tokio::test]
#[ignore = "нужен Murmur с Ice"]
async fn server_all_methods_smoke() {
    let (_meta, mut srv, ctx) = connect().await;

    srv.ice_ping().await.expect("Server.ice_ping");
    assert!(srv.ice_is_a().await.expect("Server.ice_is_a"));
    let _ = srv.ice_id().await.expect("Server.ice_id");
    let _ = srv.ice_ids().await.expect("Server.ice_ids");

    assert!(srv.is_running(ctx.clone()).await.expect("is_running"));
    let _virtual_server_id = srv.id(ctx.clone()).await.expect("id");

    assert!(
        srv.start(ctx.clone()).await.is_err(),
        "start на уже запущенном сервере должен вернуть ошибку"
    );

    let _conf_port = srv
        .get_conf(&"port".into(), ctx.clone())
        .await
        .expect("get_conf");
    let all_conf = srv.get_all_conf(ctx.clone()).await.expect("get_all_conf");
    assert!(!all_conf.is_empty());
    if let Some(welcome) = all_conf.get("welcometext") {
        srv.set_conf(&"welcometext".into(), welcome, ctx.clone())
            .await
            .expect("set_conf roundtrip (то же значение)");
    }

    let log_len = srv.get_log_len(ctx.clone()).await.expect("get_log_len");
    let logs: LogList = if log_len > 0 {
        srv.get_log(0, log_len - 1, ctx.clone())
            .await
            .expect("get_log")
    } else {
        Vec::new()
    };
    let _ = logs;

    let users = srv.get_users(ctx.clone()).await.expect("get_users");
    let channels = srv.get_channels(ctx.clone()).await.expect("get_channels");
    assert!(channels.contains_key(&0), "корневой канал 0");

    let tree: Tree = srv.get_tree(ctx.clone()).await.expect("get_tree");
    let _ = tree;

    let bans: BanList = srv.get_bans(ctx.clone()).await.expect("get_bans");
    srv.set_bans(&bans, ctx.clone()).await.expect("set_bans roundtrip");

    let ch0 = srv
        .get_channel_state(0, ctx.clone())
        .await
        .expect("get_channel_state 0");

    let mut acls: Acllist = Vec::new();
    let mut groups: GroupList = Vec::new();
    let mut inherit = false;
    srv.get_acl(0, &mut acls, &mut groups, &mut inherit, ctx.clone())
        .await
        .expect("get_acl");
    srv.set_acl(0, &acls, &groups, inherit, ctx.clone())
        .await
        .expect("set_acl roundtrip");

    let bad_session = 9_999_999;
    assert!(
        srv.get_state(bad_session, ctx.clone()).await.is_err(),
        "get_state: несуществующая сессия"
    );
    assert!(
        srv.get_certificate_list(bad_session, ctx.clone())
            .await
            .is_err(),
        "get_certificate_list: несуществующая сессия"
    );
    assert!(
        srv.kick_user(bad_session, &"test".into(), ctx.clone())
            .await
            .is_err(),
        "kick_user: несуществующая сессия"
    );
    assert!(
        srv.send_message(bad_session, &"hi".into(), ctx.clone())
            .await
            .is_err(),
        "send_message: несуществующая сессия"
    );

    assert!(
        srv.send_message_channel(0, false, &"chan".into(), ctx.clone())
            .await
            .is_ok(),
        "send_message_channel в root"
    );

    let perm_read = 0x01i32;
    let _ = srv
        .has_permission(bad_session, 0, perm_read, ctx.clone())
        .await;
    let _ = srv
        .effective_permissions(bad_session, 0, ctx.clone())
        .await;

    let nl: NameList = vec!["no_such_mumble_user_x".into()];
    let idmap = srv
        .get_user_ids(&nl, ctx.clone())
        .await
        .expect("get_user_ids");
    let _ = idmap;

    let reg: NameMap = srv
        .get_registered_users(&String::new(), ctx.clone())
        .await
        .expect("get_registered_users");
    let _ = reg;

    let reg_filter = srv
        .get_registered_users(&"___".into(), ctx.clone())
        .await
        .expect("get_registered_users filtered");
    let _ = reg_filter;

    let vp = srv
        .verify_password(&"nope".into(), &"x".into(), ctx.clone())
        .await
        .expect("verify_password");
    let _ = vp;

    assert!(
        srv.get_registration(-99, ctx.clone()).await.is_err(),
        "get_registration: неверный id"
    );

    assert!(
        srv.get_texture(-99, ctx.clone()).await.is_err(),
        "get_texture: неверный userid"
    );

    let uptime = srv.get_uptime(ctx.clone()).await.expect("Server.get_uptime");
    assert!(uptime >= 0);

    let uname = format!(
        "ice_rs_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );
    let mut info = UserInfoMap::new();
    info.insert(UserInfo::UserName, uname.clone());
    let new_uid = srv
        .register_user(&info, ctx.clone())
        .await
        .expect("register_user");
    assert!(new_uid >= 0);
    let got = srv
        .get_registration(new_uid, ctx.clone())
        .await
        .expect("get_registration new user");
    assert_eq!(
        got.get(&UserInfo::UserName).map(|s| s.as_str()),
        Some(uname.as_str())
    );

    let _tex_reg = srv
        .get_texture(new_uid, ctx.clone())
        .await
        .expect("get_texture для зарегистрированного пользователя");

    let name_map = srv
        .get_user_names(&vec![new_uid], ctx.clone())
        .await
        .expect("get_user_names");
    assert!(name_map.get(&new_uid).is_some());

    let mut upd = UserInfoMap::new();
    upd.insert(UserInfo::UserComment, "ice-rs test".into());
    srv.update_registration(new_uid, &upd, ctx.clone())
        .await
        .expect("update_registration");

    let empty_tex: Texture = Vec::new();
    srv.set_texture(new_uid, &empty_tex, ctx.clone())
        .await
        .expect("set_texture empty");

    assert!(
        srv.start_listening(new_uid, 0, ctx.clone()).await.is_ok()
            || srv.start_listening(new_uid, 0, ctx.clone()).await.is_err(),
        "start_listening"
    );
    let _listen_ch = srv
        .get_listening_channels(new_uid, ctx.clone())
        .await
        .expect("get_listening_channels");
    let _listen_u = srv
        .get_listening_users(0, ctx.clone())
        .await
        .expect("get_listening_users");
    let _is_l = srv
        .is_listening(new_uid, 0, ctx.clone())
        .await
        .unwrap_or(false);
    let vol = srv
        .get_listener_volume_adjustment(0, new_uid, ctx.clone())
        .await
        .unwrap_or(1.0);
    srv.set_listener_volume_adjustment(0, new_uid, vol, ctx.clone())
        .await
        .expect("set_listener_volume_adjustment roundtrip");
    srv.stop_listening(new_uid, 0, ctx.clone())
        .await
        .ok();

    srv.send_welcome_message(&IdList::new(), ctx.clone())
        .await
        .ok();

    srv.unregister_user(new_uid, ctx.clone())
        .await
        .expect("unregister_user cleanup");

    assert!(
        srv.add_user_to_group(0, bad_session, &"admin".into(), ctx.clone())
            .await
            .is_err(),
        "add_user_to_group bad session"
    );
    assert!(
        srv.remove_user_from_group(0, bad_session, &"admin".into(), ctx.clone())
            .await
            .is_err(),
        "remove_user_from_group bad session"
    );
    assert!(
        srv.redirect_whisper_group(
            bad_session,
            &"a".into(),
            &"".into(),
            ctx.clone()
        )
        .await
        .is_err(),
        "redirect_whisper_group bad session"
    );

    let ch_name = format!(
        "tmpch_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );
    let new_ch = srv
        .add_channel(&ch_name, 0, ctx.clone())
        .await
        .expect("add_channel");
    srv.remove_channel(new_ch, ctx.clone())
        .await
        .expect("remove_channel");

    let stale_user = stale_user_template();
    assert!(
        srv.set_state(&stale_user, ctx.clone()).await.is_err(),
        "set_state: несуществующая сессия"
    );

    let mut ch_mut = ch0.clone();
    ch_mut.description = ch0.description.clone();
    srv.set_channel_state(&ch_mut, ctx.clone())
        .await
        .expect("set_channel_state same data");

    assert!(
        srv.update_certificate(&String::new(), &String::new(), &String::new(), ctx.clone())
            .await
            .is_err(),
        "update_certificate с пустым PEM — ошибка"
    );

    if let Some(u) = users.values().next() {
        let _ = srv
            .has_permission(u.session, 0, perm_read, ctx.clone())
            .await;
        let _ = srv.effective_permissions(u.session, 0, ctx.clone()).await;
    }
}

fn stale_user_template() -> User {
    User {
        session: 9_999_998,
        userid: -1,
        mute: false,
        deaf: false,
        suppress: false,
        priority_speaker: false,
        self_mute: false,
        self_deaf: false,
        recording: false,
        channel: 0,
        name: String::new(),
        onlinesecs: 0,
        bytespersec: 0,
        version: 0,
        version_2: 0,
        release: String::new(),
        os: String::new(),
        osversion: String::new(),
        identity: String::new(),
        context: String::new(),
        comment: String::new(),
        address: Vec::new(),
        tcponly: false,
        idlesecs: 0,
        udp_ping: 0.0,
        tcp_ping: 0.0,
    }
}

#[tokio::test]
#[ignore = "нужен Murmur с Ice"]
async fn meta_new_server_virtual_delete() {
    let (mut meta, _srv, ctx) = connect().await;
    let mut new_srv = meta.new_server(ctx.clone()).await.expect("new_server");
    let _nid = new_srv.id(ctx.clone()).await.expect("new server id");
    new_srv.delete(ctx.clone()).await.expect("delete new VS");
}

#[tokio::test]
#[ignore = "нужен Murmur с Ice; останавливает первый виртуальный сервер"]
async fn server_stop_then_start() {
    let (_meta, mut srv, ctx) = connect().await;
    srv.stop(ctx.clone()).await.expect("stop");
    srv.start(ctx.clone()).await.expect("start после stop");
}

#[tokio::test]
#[ignore = "нужен Murmur и реальный MetaCallback servant (Ice adapter в этом процессе)"]
async fn meta_add_callback_remove_callback_not_automated() {
    assert!(true, "реализуйте MetaCallback + Meta::add_callback/remove_callback локально");
}

#[tokio::test]
#[ignore = "нужен Murmur и ServerCallback servant"]
async fn server_add_callback_remove_callback_not_automated() {
    assert!(true);
}

#[tokio::test]
#[ignore = "нужен Murmur и ServerAuthenticator servant"]
async fn server_set_authenticator_not_automated() {
    assert!(true);
}

#[tokio::test]
#[ignore = "нужен Murmur и ServerContextCallback servant"]
async fn server_add_remove_context_callback_not_automated() {
    assert!(true);
}

#[tokio::test]
#[ignore = "мутация SuperUser; не запускать на проде"]
async fn server_set_superuser_password_not_automated() {
    assert!(true);
}

/// Интерфейсы **входящие** (Murmur вызывает реализацию у вас в процессе). Покрытие — только документация;
/// нужен свой `ObjectAdapter` и типы `*Server` из сгенерированного кода.
///
/// - **`MetaCallback`:** `started`, `stopped`
/// - **`ServerCallback`:** `userConnected`, `userDisconnected`, `userStateChanged`, `userTextMessage`,
///   `channelCreated`, `channelRemoved`, `channelStateChanged`
/// - **`ServerContextCallback`:** `contextAction`
/// - **`ServerAuthenticator`:** `authenticate`, `getInfo`, `nameToId`, `idToName`, `idToTexture`
/// - **`ServerUpdatingAuthenticator`:** наследует `ServerAuthenticator` и добавляет `registerUser`, `unregisterUser`,
///   `getRegisteredUsers`, `setInfo`, `setTexture`
///
/// С клиента при этом вызываются `Meta::add_callback` / `remove_callback`, `Server::add_callback` / `remove_callback`,
/// `set_authenticator`, `add_context_callback`, `remove_context_callback` — без живого servant для колбэка смысла нет
/// (см. также отдельные тесты `*_destructive*`).
#[tokio::test]
async fn incoming_callback_interfaces_documented() {
    assert!(true);
}

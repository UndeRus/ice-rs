//! Тесты `ServerCallback`: сообщения, дерево каналов, пользователи.
//!
//! - **Модульные** (`*_dispatch`) — вызывают `ServerCallbackServer::handle_request` с синтетическим `RequestData` (без Murmur).
//! - **Интеграционные** — Murmur вызывает колбэки по TCP; запуск:  
//!   `cargo test -p mumble-ice-demo --test server_callback_tests -- --ignored`
//!
//! Для сообщений от пользователя нужна **живая Mumble-сессия** на сервере; иначе тест пропускается с сообщением в stderr.

use async_trait::async_trait;
use ice_rs::adapter::Adapter;
use ice_rs::communicator::Communicator;
use ice_rs::encoding::{FromBytes, ToBytes};
use ice_rs::iceobject::IceObjectServer;
use ice_rs::protocol::{Encapsulation, Identity, RequestData};
use mumble_ice_demo::gen::mumble_server::{
    Channel, Meta, MetaPrx, Server, ServerCallbackI, ServerCallbackPrx, ServerCallbackServer, TextMessage,
    User,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const CB_IDENT: &str = "IceRsCb";

type IceCtx = Option<HashMap<String, String>>;

#[derive(Debug, Clone, PartialEq)]
enum CbEvt {
    UserConnected(i32),
    UserDisconnected(i32),
    UserStateChanged(i32),
    UserTextMessage { session: i32, text: String },
    ChannelCreated(i32),
    ChannelRemoved(i32),
    ChannelStateChanged(i32),
}

struct Recorder {
    ev: Arc<Mutex<Vec<CbEvt>>>,
}

#[async_trait]
impl ServerCallbackI for Recorder {
    async fn user_connected(
        &mut self,
        state: &User,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::UserConnected(state.session));
    }

    async fn user_disconnected(
        &mut self,
        state: &User,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::UserDisconnected(state.session));
    }

    async fn user_state_changed(
        &mut self,
        state: &User,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::UserStateChanged(state.session));
    }

    async fn user_text_message(
        &mut self,
        state: &User,
        message: &TextMessage,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::UserTextMessage {
            session: state.session,
            text: message.text.clone(),
        });
    }

    async fn channel_created(
        &mut self,
        state: &Channel,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::ChannelCreated(state.id));
    }

    async fn channel_removed(
        &mut self,
        state: &Channel,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev.lock().await.push(CbEvt::ChannelRemoved(state.id));
    }

    async fn channel_state_changed(
        &mut self,
        state: &Channel,
        _ctx: Option<HashMap<String, String>>,
    ) {
        self.ev
            .lock()
            .await
            .push(CbEvt::ChannelStateChanged(state.id));
    }
}

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

fn sample_user(session: i32) -> User {
    User {
        session,
        userid: -1,
        mute: false,
        deaf: false,
        suppress: false,
        priority_speaker: false,
        self_mute: false,
        self_deaf: false,
        recording: false,
        channel: 0,
        name: "unit".into(),
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

fn sample_channel(id: i32) -> Channel {
    Channel {
        id,
        name: format!("ch{}", id),
        parent: 0,
        links: Vec::new(),
        description: String::new(),
        temporary: false,
        position: 0,
    }
}

fn req(op: &str, params: Vec<u8>) -> RequestData {
    RequestData {
        request_id: 1,
        id: Identity::new(CB_IDENT),
        facet: Vec::new(),
        operation: op.into(),
        mode: 1,
        context: HashMap::new(),
        params: Encapsulation::from(params),
    }
}

fn spawn_callback_listener(
    port: u16,
    recorder: Arc<Mutex<Vec<CbEvt>>>,
) -> tokio::task::JoinHandle<()> {
    let mut adapter =
        Adapter::with_endpoint(CB_IDENT, &format!("tcp -h 127.0.0.1 -p {}", port)).unwrap();
    adapter.add(
        CB_IDENT,
        Box::new(ServerCallbackServer::new(Box::new(Recorder {
            ev: recorder,
        }))),
    );
    let adapter = Arc::new(Mutex::new(adapter));
    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await.expect("bind callback port");
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                continue;
            };
            let adapter = adapter.clone();
            tokio::spawn(async move {
                let _ = adapter.lock().await.handle_socket(&mut socket).await;
            });
        }
    })
}

async fn wait_for_event(
    ev: &Arc<Mutex<Vec<CbEvt>>>,
    pred: impl Fn(&CbEvt) -> bool,
    timeout_ms: u64,
) -> bool {
    let deadline = std::time::Duration::from_millis(timeout_ms);
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        {
            let g = ev.lock().await;
            if g.iter().any(&pred) {
                return true;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    false
}

// --- Модульные тесты (без Murmur) ---

#[tokio::test]
async fn callback_dispatch_ice_object_ops() {
    let ev = Arc::new(Mutex::new(Vec::new()));
    let mut srv = ServerCallbackServer::new(Box::new(Recorder { ev: ev.clone() }));
    let type_str = String::from("::MumbleServer::ServerCallback");

    // Как у Murmur: строка в одной или двух Encapsulation внутри params.data
    let inner = Encapsulation::from(type_str.to_bytes().unwrap());
    let double = Encapsulation::from(inner.to_bytes().unwrap())
        .to_bytes()
        .unwrap();
    let r = req("ice_isA", double);
    let reply = srv.handle_request(&r).await.unwrap();
    let mut rb: i32 = 0;
    assert!(bool::from_bytes(&reply.body.data, &mut rb).unwrap());

    let r2 = req("ice_isA", type_str.to_bytes().unwrap());
    let reply2 = srv.handle_request(&r2).await.unwrap();
    rb = 0;
    assert!(bool::from_bytes(&reply2.body.data, &mut rb).unwrap());

    srv.handle_request(&req("ice_ping", vec![])).await.unwrap();

    let reply4 = srv
        .handle_request(&req("ice_id", vec![]))
        .await
        .unwrap();
    rb = 0;
    let id = String::from_bytes(&reply4.body.data, &mut rb).unwrap();
    assert_eq!(id, "::MumbleServer::ServerCallback");

    let reply5 = srv
        .handle_request(&req("ice_ids", vec![]))
        .await
        .unwrap();
    rb = 0;
    let ids: Vec<String> = Vec::from_bytes(&reply5.body.data, &mut rb).unwrap();
    assert_eq!(
        ids,
        vec![
            "::MumbleServer::ServerCallback".to_string(),
            "::Ice::Object".to_string(),
        ]
    );
}

#[tokio::test]
async fn callback_dispatch_user_connected() {
    let ev = Arc::new(Mutex::new(Vec::new()));
    let mut srv = ServerCallbackServer::new(Box::new(Recorder { ev: ev.clone() }));
    let u = sample_user(42);
    let r = req("userConnected", u.to_bytes().unwrap());
    srv.handle_request(&r).await.unwrap();
    assert!(ev.lock().await.contains(&CbEvt::UserConnected(42)));
}

#[tokio::test]
async fn callback_dispatch_user_text_message() {
    let ev = Arc::new(Mutex::new(Vec::new()));
    let mut srv = ServerCallbackServer::new(Box::new(Recorder { ev: ev.clone() }));
    let u = sample_user(7);
    let msg = TextMessage {
        sessions: vec![],
        channels: vec![],
        trees: vec![],
        text: "hello from ice-rs test".into(),
    };
    let mut params = Vec::new();
    params.extend(u.to_bytes().unwrap());
    params.extend(msg.to_bytes().unwrap());
    let r = req("userTextMessage", params);
    srv.handle_request(&r).await.unwrap();
    assert!(
        ev.lock().await.contains(&CbEvt::UserTextMessage {
            session: 7,
            text: "hello from ice-rs test".into(),
        })
    );
}

#[tokio::test]
async fn callback_dispatch_channel_created_removed_changed() {
    let ev = Arc::new(Mutex::new(Vec::new()));
    let mut srv = ServerCallbackServer::new(Box::new(Recorder { ev: ev.clone() }));
    for (op, id, expect) in [
        (
            "channelCreated",
            100,
            CbEvt::ChannelCreated(100),
        ),
        ("channelRemoved", 100, CbEvt::ChannelRemoved(100)),
        (
            "channelStateChanged",
            5,
            CbEvt::ChannelStateChanged(5),
        ),
    ] {
        let r = req(op, sample_channel(id).to_bytes().unwrap());
        srv.handle_request(&r).await.unwrap();
        assert!(ev.lock().await.contains(&expect), "op {}", op);
    }
}

#[tokio::test]
async fn callback_dispatch_user_state_changed() {
    let ev = Arc::new(Mutex::new(Vec::new()));
    let mut srv = ServerCallbackServer::new(Box::new(Recorder { ev: ev.clone() }));
    let u = sample_user(99);
    let r = req("userStateChanged", u.to_bytes().unwrap());
    srv.handle_request(&r).await.unwrap();
    assert!(ev.lock().await.contains(&CbEvt::UserStateChanged(99)));
}

// --- Интеграция с Murmur ---

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "нужен Murmur с Ice на 127.0.0.1:6502"]
async fn integration_channel_tree_callbacks() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let ev = Arc::new(Mutex::new(Vec::new()));
    let _bg = spawn_callback_listener(port, ev.clone());
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let mut comm = Communicator::new().await.expect("comm");
    let ctx = ice_context();
    let proxy = comm
        .string_to_proxy(&endpoint_string())
        .await
        .expect("meta proxy");
    let mut meta = MetaPrx::unchecked_cast(proxy).await.expect("meta cast");
    let mut servers = meta
        .get_booted_servers(ctx.clone())
        .await
        .expect("booted");
    let mut srv = servers.pop().expect("vs");

    let cb_str = format!("{}:tcp -h 127.0.0.1 -p {}", CB_IDENT, port);
    let cb_proxy = comm.string_to_proxy(&cb_str).await.expect("cb proxy");
    let cb = ServerCallbackPrx::unchecked_cast(cb_proxy)
        .await
        .expect("cb cast");
    srv.add_callback(&cb, ctx.clone()).await.expect("add_callback");

    let tag = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let ch_name = format!("ice_rs_cb_{}", tag);
    let ch_id = srv
        .add_channel(&ch_name, 0, ctx.clone())
        .await
        .expect("add_channel");

    assert!(
        wait_for_event(&ev, |e| matches!(e, CbEvt::ChannelCreated(id) if *id == ch_id), 8000).await,
        "ожидали channelCreated id={}, события: {:?}",
        ch_id,
        ev.lock().await
    );

    let mut st = srv
        .get_channel_state(ch_id, ctx.clone())
        .await
        .expect("get_channel_state");
    let old_desc = st.description.clone();
    st.description = format!("tmp-{}", tag);
    srv.set_channel_state(&st, ctx.clone())
        .await
        .expect("set_channel_state");
    assert!(
        wait_for_event(&ev, |e| matches!(e, CbEvt::ChannelStateChanged(id) if *id == ch_id), 8000).await,
        "ожидали channelStateChanged, события: {:?}",
        ev.lock().await
    );
    st.description = old_desc;
    srv.set_channel_state(&st, ctx.clone()).await.ok();

    srv.remove_channel(ch_id, ctx.clone())
        .await
        .expect("remove_channel");
    assert!(
        wait_for_event(&ev, |e| matches!(e, CbEvt::ChannelRemoved(id) if *id == ch_id), 8000).await,
        "ожидали channelRemoved, события: {:?}",
        ev.lock().await
    );

    srv.remove_callback(&cb, ctx.clone()).await.expect("remove_callback");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "нужен Murmur с Ice; для userTextMessage — подключённый Mumble-клиент"]
async fn integration_user_and_message_callbacks() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let ev = Arc::new(Mutex::new(Vec::new()));
    let _bg = spawn_callback_listener(port, ev.clone());
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let mut comm = Communicator::new().await.expect("comm");
    let ctx = ice_context();
    let proxy = comm
        .string_to_proxy(&endpoint_string())
        .await
        .expect("meta");
    let mut meta = MetaPrx::unchecked_cast(proxy).await.expect("meta");
    let mut servers = meta
        .get_booted_servers(ctx.clone())
        .await
        .expect("booted");
    let mut srv = servers.pop().expect("vs");

    let cb_str = format!("{}:tcp -h 127.0.0.1 -p {}", CB_IDENT, port);
    let cb_proxy = comm.string_to_proxy(&cb_str).await.expect("cb");
    let cb = ServerCallbackPrx::unchecked_cast(cb_proxy).await.expect("cast");
    srv.add_callback(&cb, ctx.clone()).await.expect("add_callback");

    let users = srv.get_users(ctx.clone()).await.expect("get_users");
    if users.is_empty() {
        eprintln!("integration_user_and_message_callbacks: нет подключённых пользователей — user* / userTextMessage не проверяются");
        srv.remove_callback(&cb, ctx.clone()).await.ok();
        return;
    }

    let u = users.values().next().unwrap().clone();
    let session = u.session;

    srv.send_message(
        session,
        &"ice-rs callback probe".into(),
        ctx.clone(),
    )
    .await
    .expect("send_message");

    let got_msg = wait_for_event(
        &ev,
        |e| {
            matches!(e, CbEvt::UserTextMessage { text, .. } if text.contains("ice-rs callback probe"))
        },
        5000,
    )
    .await;
    assert!(
        got_msg,
        "ожидали userTextMessage после send_message, события: {:?}",
        ev.lock().await
    );

    let mut u2 = u.clone();
    u2.comment = format!(
        "ice-rs-cb-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    srv.set_state(&u2, ctx.clone()).await.expect("set_state");

    assert!(
        wait_for_event(&ev, |e| matches!(e, CbEvt::UserStateChanged(s) if *s == session), 5000).await,
        "ожидали userStateChanged после set_state, события: {:?}",
        ev.lock().await
    );

    srv.remove_callback(&cb, ctx.clone()).await.expect("remove_callback");
}

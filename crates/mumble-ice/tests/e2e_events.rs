//! E2E колбеков против живого Murmur.
//!
//! ```sh
//! cargo nextest run -p mumble-ice --test e2e_events --run-ignored all
//! ```
//!
//! Murmur 1.5.857 **не рассылает** `channelCreated` для каналов, созданных через
//! Ice, но рассылает `channelStateChanged` и `channelRemoved` — проверено
//! отдельным пробником. Поэтому триггерим именно их.

use async_trait::async_trait;
use mumble_ice::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

fn endpoint() -> String {
    std::env::var("MUMBLE_ICE_ENDPOINT").unwrap_or_else(|_| String::from("127.0.0.1:6502"))
}

async fn connect() -> MurmurClient {
    let mut b = MurmurClient::builder()
        .endpoint(endpoint().as_str())
        .expect("адрес")
        // Слушаем на конкретном интерфейсе: Murmur должен дозвониться обратно.
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

#[derive(Debug, Clone, PartialEq)]
enum Seen {
    ChannelStateChanged(ChannelId),
    ChannelRemoved(ChannelId),
    UserConnected(String),
    Reattached,
}

struct Recorder {
    seen: Arc<Mutex<Vec<Seen>>>,
    errors: Arc<Mutex<Vec<String>>>,
    /// Если выставлено — обработчик паникует, чтобы проверить, что подписка
    /// это переживает.
    panic_once: AtomicUsize,
}

impl Recorder {
    fn new() -> (Arc<Recorder>, Arc<Mutex<Vec<Seen>>>, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        (
            Arc::new(Recorder {
                seen: seen.clone(),
                errors: errors.clone(),
                panic_once: AtomicUsize::new(0),
            }),
            seen,
            errors,
        )
    }
}

#[async_trait]
impl ServerEvents for Recorder {
    async fn user_connected(&self, _srv: &VirtualServer, u: User) -> mumble_ice::Result<()> {
        self.seen.lock().await.push(Seen::UserConnected(u.name));
        Ok(())
    }

    async fn channel_state_changed(
        &self,
        _srv: &VirtualServer,
        c: Channel,
    ) -> mumble_ice::Result<()> {
        if self.panic_once.swap(0, Ordering::AcqRel) > 0 {
            panic!("нарочная паника в обработчике");
        }
        self.seen.lock().await.push(Seen::ChannelStateChanged(c.id));
        Ok(())
    }

    async fn channel_removed(&self, _srv: &VirtualServer, c: Channel) -> mumble_ice::Result<()> {
        self.seen.lock().await.push(Seen::ChannelRemoved(c.id));
        Ok(())
    }

    async fn reattached(&self, _srv: &VirtualServer) -> mumble_ice::Result<()> {
        self.seen.lock().await.push(Seen::Reattached);
        Ok(())
    }

    async fn on_error(&self, err: mumble_ice::Error) {
        self.errors.lock().await.push(err.to_string());
    }
}

async fn wait_for<F>(seen: &Arc<Mutex<Vec<Seen>>>, pred: F, secs: u64) -> bool
where
    F: Fn(&Seen) -> bool,
{
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < deadline {
        if seen.lock().await.iter().any(&pred) {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    false
}

/// Приёмка S7: событие Murmur'а доезжает до трейта фасада.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice на 127.0.0.1:6502"]
async fn server_events_reach_the_handler() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let (rec, seen, errors) = Recorder::new();
    let sub = srv.on_events(rec).await.expect("on_events");
    assert!(sub.is_alive());

    let name = format!("mi_ev_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");

    // channelStateChanged
    srv.set_channel_description(id, &format!("d-{}", tag()))
        .await
        .expect("set_channel_description");
    assert!(
        wait_for(&seen, |e| *e == Seen::ChannelStateChanged(id), 8).await,
        "channelStateChanged не доехал; события: {:?}, ошибки: {:?}",
        seen.lock().await,
        errors.lock().await
    );

    // channelRemoved
    srv.remove_channel(id).await.expect("remove_channel");
    assert!(
        wait_for(&seen, |e| *e == Seen::ChannelRemoved(id), 8).await,
        "channelRemoved не доехал; события: {:?}",
        seen.lock().await
    );

    sub.unsubscribe().await.expect("unsubscribe");
    client.shutdown().await;
}

/// Паника в обработчике не должна убивать подписку: Murmur снимает колбек, если
/// тот бросил исключение, поэтому мы обязаны отвечать «ок» всегда.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn handler_panic_does_not_kill_the_subscription() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let (rec, seen, errors) = Recorder::new();
    rec.panic_once.store(1, Ordering::Release);
    let sub = srv.on_events(rec).await.expect("on_events");

    let name = format!("mi_panic_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");

    // Первое изменение уйдёт в панику.
    srv.set_channel_description(id, "первое").await.expect("set 1");
    let saw_error = {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        let mut got = false;
        while std::time::Instant::now() < deadline {
            if errors.lock().await.iter().any(|e| e.contains("паника")) {
                got = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        got
    };
    assert!(saw_error, "паника должна прийти в on_error, ошибки: {:?}", errors.lock().await);
    assert!(sub.is_alive(), "подписка должна выжить после паники");

    // Второе изменение должно быть обработано нормально.
    srv.set_channel_description(id, "второе").await.expect("set 2");
    assert!(
        wait_for(&seen, |e| *e == Seen::ChannelStateChanged(id), 8).await,
        "после паники подписка перестала работать; события: {:?}",
        seen.lock().await
    );

    srv.remove_channel(id).await.ok();
    sub.unsubscribe().await.ok();
    client.shutdown().await;
}

/// Две подписки на одном сервере: обе живут на одном адаптере.
///
/// Murmur открывает отдельное соединение на каждый callback-прокси, поэтому это
/// проверяет конкурентность адаптера через фасад.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn two_subscriptions_share_one_adapter() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let (rec_a, seen_a, _) = Recorder::new();
    let (rec_b, seen_b, _) = Recorder::new();
    let sub_a = srv.on_events(rec_a).await.expect("подписка A");
    let sub_b = srv.on_events(rec_b).await.expect("подписка B");

    let name = format!("mi_two_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");
    srv.set_channel_description(id, "оба").await.expect("set");

    let a = wait_for(&seen_a, |e| *e == Seen::ChannelStateChanged(id), 8).await;
    let b = wait_for(&seen_b, |e| *e == Seen::ChannelStateChanged(id), 8).await;

    srv.remove_channel(id).await.ok();
    sub_a.unsubscribe().await.ok();
    sub_b.unsubscribe().await.ok();
    client.shutdown().await;

    assert!(a, "подписка A не получила событие");
    assert!(b, "подписка B не получила событие — второй servant голодает");
}

/// Поток событий — та же машинерия, другой интерфейс.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn event_stream_delivers() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let mut stream = srv.events().await.expect("events");

    let name = format!("mi_stream_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");
    srv.set_channel_description(id, "поток").await.expect("set");

    let got = tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            match stream.recv().await {
                Some(Event::ChannelStateChanged(c)) if c.id == id => return true,
                Some(_) => continue,
                None => return false,
            }
        }
    })
    .await
    .unwrap_or(false);

    srv.remove_channel(id).await.ok();
    stream.unsubscribe().await.ok();
    client.shutdown().await;

    assert!(got, "поток не отдал ChannelStateChanged");
}

/// Снятая подписка больше не получает событий.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn unsubscribe_stops_delivery() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let (rec, seen, _) = Recorder::new();
    let sub = srv.on_events(rec).await.expect("on_events");

    let name = format!("mi_unsub_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel");
    srv.set_channel_description(id, "до").await.expect("set до");
    assert!(
        wait_for(&seen, |e| *e == Seen::ChannelStateChanged(id), 8).await,
        "до отписки событие должно приходить"
    );

    sub.unsubscribe().await.expect("unsubscribe");
    seen.lock().await.clear();

    srv.set_channel_description(id, "после").await.expect("set после");
    // Даём заведомо больше времени, чем нужно живой подписке.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let after = seen.lock().await.clone();

    srv.remove_channel(id).await.ok();
    client.shutdown().await;

    assert!(
        after.is_empty(),
        "после отписки события приходить не должны: {:?}",
        after
    );
}

/// Переподписка после перезапуска виртуального сервера.
///
/// `MumbleServer.ice` предупреждает: остановка виртуального сервера снимает
/// колбеки, и вернуть их — забота клиента. Именно это здесь и проверяется.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "деструктивно: останавливает и запускает виртуальный сервер"]
async fn reattaches_after_virtual_server_restart() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let (rec, seen, errors) = Recorder::new();
    let sub = srv.on_events(rec).await.expect("on_events");

    srv.stop().await.expect("stop");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    srv.start().await.expect("start");

    let reattached = wait_for(&seen, |e| *e == Seen::Reattached, 15).await;
    assert!(
        reattached,
        "не пришёл reattached; события: {:?}, ошибки: {:?}",
        seen.lock().await,
        errors.lock().await
    );

    // И подписка должна снова работать.
    seen.lock().await.clear();
    let name = format!("mi_re_{}", tag());
    let id = srv
        .create_channel(&name, ChannelId::ROOT)
        .await
        .expect("create_channel после перезапуска");
    srv.set_channel_description(id, "после перезапуска")
        .await
        .expect("set");
    let works = wait_for(&seen, |e| *e == Seen::ChannelStateChanged(id), 10).await;

    srv.remove_channel(id).await.ok();
    sub.unsubscribe().await.ok();
    client.shutdown().await;

    assert!(works, "после переподписки события не идут: {:?}", seen.lock().await);
}

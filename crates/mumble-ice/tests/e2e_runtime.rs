//! Свойства слоя соединения: однопоточный рантайм и конкурентность.
//!
//! ```sh
//! cargo nextest run -p mumble-ice --test e2e_runtime --run-ignored all
//! ```

use mumble_ice::prelude::*;

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

/// Подключение и вызов на **однопоточном** рантайме.
///
/// Раньше падало сразу: `Drop for Proxy` вызывал `tokio::task::block_in_place`,
/// который вне multi-thread рантайма паникует с «can call blocking only when
/// running on the multi-threaded runtime».
#[tokio::test(flavor = "current_thread")]
#[ignore = "нужен Murmur с Ice"]
async fn current_thread_runtime_works() {
    let client = connect().await;
    let v = client.version().await.expect("version");
    assert!(v.major > 0 || !v.text.is_empty(), "версия: {}", v);
    client.shutdown().await;
}

/// Приём прокси по проводу на однопоточном рантайме.
///
/// `getBootedServers` возвращает `ServerList`, то есть последовательность
/// прокси. Раньше `Proxy::from_bytes` вызывал `futures::executor::block_on` и
/// открывал TCP-соединение **внутри десериализации** — декодер блокировал
/// реактор, которого сам же и ждал, и тест висел до таймаута (замерено: 58 с
/// против 0.05 с сейчас).
#[tokio::test(flavor = "current_thread")]
#[ignore = "нужен Murmur с Ice"]
async fn decoding_a_proxy_does_not_deadlock_on_current_thread() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");
    assert!(srv.is_running().await.expect("is_running"));
    client.shutdown().await;
}

/// Несколько вызовов в полёте по одному соединению.
///
/// Раньше `dispatch` брал `&mut self`, то есть один запрос за раз — и это было
/// навязано borrow checker'ом, а не транспортом.
#[tokio::test(flavor = "current_thread")]
#[ignore = "нужен Murmur с Ice"]
async fn calls_are_concurrent_over_one_connection() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    // Десять вызовов одновременно с ОДНОГО таска на однопоточном рантайме:
    // сериализованный слой такого не смог бы вовсе.
    let started = std::time::Instant::now();
    let (a, b, c, d, e) = tokio::join!(
        srv.users(),
        srv.channels(),
        srv.all_config(),
        srv.log_len(),
        srv.uptime(),
    );
    let elapsed = started.elapsed();

    a.expect("users");
    b.expect("channels");
    c.expect("all_config");
    d.expect("log_len");
    e.expect("uptime");

    // Доказательство фактом, а не временем: соединение видело больше одного
    // запроса в полёте одновременно. Замер времени на локальном сервере тонет в
    // шуме таймера.
    let peak = srv.max_in_flight().await;
    println!("[конкурентность] пять вызовов за {:?}, пик в полёте: {}", elapsed, peak);
    assert!(
        peak > 1,
        "вызовы сериализовались: пик одновременных запросов {}",
        peak
    );
    client.shutdown().await;
}

/// Хендл шарится между тасками без внешнего мьютекса.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn handle_is_shareable_and_concurrent() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    let mut tasks = Vec::new();
    for i in 0..8 {
        let s = srv.clone();
        tasks.push(tokio::spawn(async move {
            // Перемежаем разные операции, чтобы ответы точно шли не по порядку.
            if i % 2 == 0 {
                s.users().await.map(|u| u.len())
            } else {
                s.channels().await.map(|c| c.len())
            }
        }));
    }
    for (i, t) in tasks.into_iter().enumerate() {
        t.await
            .expect("таск не паникует")
            .unwrap_or_else(|e| panic!("вызов {} упал: {}", i, e));
    }
    client.shutdown().await;
}

/// Ответы должны попадать ровно тому, кто их ждёт.
///
/// Проверяем содержимым, а не только успехом: если корреляция по request_id
/// сломана, вызовы завершатся, но с чужими ответами.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice"]
async fn concurrent_replies_are_not_mixed_up() {
    let client = connect().await;
    let srv = client.only_server().await.expect("only_server");

    // Создаём три канала с разными именами и параллельно перечитываем каждый.
    let tag = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let mut made = Vec::new();
    for i in 0..3 {
        let name = format!("mi_rt_{}_{}", tag, i);
        let id = srv
            .create_channel(&name, ChannelId::ROOT)
            .await
            .expect("create_channel");
        made.push((id, name));
    }

    let mut tasks = Vec::new();
    for (id, name) in made.clone() {
        let s = srv.clone();
        tasks.push(tokio::spawn(async move {
            let ch = s.channel(id).await.expect("channel");
            (ch.id, ch.name, id, name)
        }));
    }

    let mut mismatches = Vec::new();
    for t in tasks {
        let (got_id, got_name, want_id, want_name) = t.await.expect("таск");
        if got_id != want_id || got_name != want_name {
            mismatches.push(format!(
                "ждали {:?}/{}, получили {:?}/{}",
                want_id, want_name, got_id, got_name
            ));
        }
    }

    for (id, _) in made {
        srv.remove_channel(id).await.ok();
    }
    client.shutdown().await;

    assert!(
        mismatches.is_empty(),
        "ответы перепутаны между вызовами: {:?}",
        mismatches
    );
}

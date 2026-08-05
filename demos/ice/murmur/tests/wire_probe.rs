//! Диагностический пробник: показывает, что именно Murmur присылает на
//! callback-эндпоинт. Нужен, чтобы отличить «Murmur не дозвонился» от
//! «дозвонился, но мы не сдиспатчили».
//!
//! Запуск: `cargo nextest run --test wire_probe --run-ignored all --no-capture`

use ice_rs::communicator::Communicator;
use ice_rs::encoding::FromBytes;
use ice_rs::protocol::{Header, RequestData};
use murmur_slice::mumble_server::{Meta, MetaPrx, Server, ServerCallbackPrx};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

fn endpoint_string() -> String {
    std::env::var("MUMBLE_ICE_ENDPOINT")
        .unwrap_or_else(|_| String::from("Meta:tcp -h 127.0.0.1 -p 6502"))
}

fn ice_context() -> Option<HashMap<String, String>> {
    match std::env::var("MUMBLE_ICE_SECRET") {
        Ok(s) if !s.is_empty() => {
            let mut m = HashMap::new();
            m.insert(String::from("secret"), s);
            Some(m)
        }
        _ => None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "нужен Murmur с Ice на 127.0.0.1:6502"]
async fn probe_what_murmur_sends_to_a_callback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().unwrap().port();
    println!("[probe] слушаем 127.0.0.1:{}", port);

    let frames: Arc<Mutex<Vec<(u8, Option<String>, i32)>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = frames.clone();

    tokio::spawn(async move {
        loop {
            let (mut sock, peer) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    println!("[probe] accept error: {}", e);
                    continue;
                }
            };
            println!("[probe] ВХОДЯЩЕЕ СОЕДИНЕНИЕ от {}", peer);
            let sink = sink.clone();
            tokio::spawn(async move {
                // Murmur ждёт ValidateConnection перед тем, как что-то послать.
                use ice_rs::encoding::ToBytes;
                use tokio::io::AsyncWriteExt;
                // Эксперимент: если PROBE_NO_VALIDATE выставлен, намеренно НЕ
                // посылаем ValidateConnection. Ice запрещает клиенту слать
                // запросы до него, так что это воспроизводит ситуацию, когда
                // адаптер занят другим соединением и не успел поздороваться.
                if std::env::var("PROBE_NO_VALIDATE").is_ok() {
                    println!("[probe] ValidateConnection НЕ послан (эксперимент)");
                } else {
                    let hello = Header::new(3, 14).to_bytes().unwrap();
                    if let Err(e) = sock.write_all(&hello).await {
                        println!("[probe] не смогли послать ValidateConnection: {}", e);
                        return;
                    }
                    let _ = sock.flush().await;
                    println!("[probe] послали ValidateConnection");
                }

                loop {
                    let mut hdr = [0u8; 14];
                    if let Err(e) = sock.read_exact(&mut hdr).await {
                        println!("[probe] соединение закрылось: {}", e);
                        return;
                    }
                    let mut r = 0i32;
                    let header = match Header::from_bytes(&hdr, &mut r) {
                        Ok(h) => h,
                        Err(e) => {
                            println!("[probe] битый заголовок: {}", e);
                            return;
                        }
                    };
                    let rest = (header.message_size - 14).max(0) as usize;
                    let mut body = vec![0u8; rest];
                    if rest > 0 {
                        if let Err(e) = sock.read_exact(&mut body).await {
                            println!("[probe] обрыв в теле: {}", e);
                            return;
                        }
                    }
                    println!(
                        "[probe] кадр: type={} size={} body={} байт",
                        header.message_type, header.message_size, rest
                    );
                    let mut op = None;
                    let mut req_id = -1;
                    if header.message_type == 0 {
                        let mut rr = 0i32;
                        match RequestData::from_bytes(&body, &mut rr) {
                            Ok(req) => {
                                println!(
                                    "[probe]   -> ЗАПРОС id={} identity={}/{} operation={} mode={}",
                                    req.request_id,
                                    req.id.category,
                                    req.id.name,
                                    req.operation,
                                    req.mode
                                );
                                op = Some(req.operation.clone());
                                req_id = req.request_id;
                            }
                            Err(e) => println!("[probe]   -> запрос не разобрался: {}", e),
                        }
                    }
                    sink.lock().await.push((header.message_type, op, req_id));
                }
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let mut comm = Communicator::new().await.expect("comm");
    let ctx = ice_context();
    let proxy = comm.string_to_proxy(&endpoint_string()).await.expect("meta");
    let mut meta = MetaPrx::unchecked_cast(proxy).await.expect("cast");
    let mut servers = meta.get_booted_servers(ctx.clone()).await.expect("booted");
    let mut srv = servers.pop().expect("нужен запущенный виртуальный сервер");

    let cb_str = format!("IceRsProbe:tcp -h 127.0.0.1 -p {}", port);
    let cb_proxy = comm.string_to_proxy(&cb_str).await.expect("cb proxy");
    let cb = ServerCallbackPrx::unchecked_cast(cb_proxy).await.expect("cb cast");
    srv.add_callback(&cb, ctx.clone()).await.expect("add_callback");
    println!("[probe] add_callback принят Murmur'ом");

    let tag = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let name = format!("probe_{}", tag);
    let ch = srv.add_channel(&name, 0, ctx.clone()).await.expect("add_channel");
    println!("[probe] создали канал id={}", ch);

    // Ждём каждое событие отдельно и с равным дедлайном, иначе «не приехало»
    // невозможно отличить от «приехало позже проверки».
    async fn wait_for_new(
        frames: &Arc<Mutex<Vec<(u8, Option<String>, i32)>>>,
        at_least: usize,
        secs: u64,
    ) -> Vec<(u8, Option<String>, i32)> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        while std::time::Instant::now() < deadline {
            if frames.lock().await.len() > at_least {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        frames.lock().await.clone()
    }

    let after_create = wait_for_new(&frames, 0, 6).await;
    println!("[probe] после addChannel: {:?}", after_create);

    let mut st = srv
        .get_channel_state(ch, ctx.clone())
        .await
        .expect("get_channel_state");
    st.description = format!("probe-desc-{}", tag);
    srv.set_channel_state(&st, ctx.clone())
        .await
        .expect("set_channel_state");
    let after_state = wait_for_new(&frames, after_create.len(), 6).await;
    println!("[probe] после setChannelState: {:?}", after_state);

    srv.remove_channel(ch, ctx.clone()).await.ok();
    let after_remove = wait_for_new(&frames, after_state.len(), 6).await;
    println!("[probe] после removeChannel: {:?}", after_remove);

    srv.remove_callback(&cb, ctx.clone()).await.ok();

    let seen = |op: &str| -> bool {
        after_remove
            .iter()
            .any(|(_, o, _)| o.as_deref() == Some(op))
    };
    println!(
        "[probe] ВЫВОД: channelCreated={} channelStateChanged={} channelRemoved={}",
        seen("channelCreated"),
        seen("channelStateChanged"),
        seen("channelRemoved")
    );

    // Приёмка S1 — транспортная: Murmur разобрал наш прокси, дозвонился и
    // прислал разбираемый Ice-запрос нашему servant'у. Какие именно события
    // Murmur считает нужным рассылать — вопрос его семантики, не нашего провода.
    assert!(
        !after_remove.is_empty(),
        "Murmur не прислал ни одного запроса на callback-эндпоинт"
    );
    assert!(
        after_remove.iter().any(|(t, op, _)| *t == 0 && op.is_some()),
        "кадры пришли, но ни один не разобрался как Ice-запрос: {:?}",
        after_remove
    );
}

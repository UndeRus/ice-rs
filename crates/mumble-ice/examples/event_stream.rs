//! Поток событий: для ботов, чей главный цикл уже `select!`-ится на чём-то ещё.
//!
//! ```sh
//! cargo run -p mumble-ice --example event_stream
//! ```
//!
//! Это производная от [`ServerEvents`], а не вторая реализация: под капотом тот
//! же мост, просто события уезжают в канал.

use mumble_ice::prelude::*;

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

    // Ёмкость и политика переполнения — явные. По умолчанию 256 и DropNewest:
    // потери считаются и приезжают как `Event::Lagged`, а не проглатываются.
    let mut events = srv.events_with(64, Overflow::DropNewest).await?;
    println!("слушаем события сервера {}; Ctrl-C для выхода", srv.id());

    // Свой таймер — чтобы показать, зачем вообще поток: он спокойно живёт в
    // одном `select!` с чем угодно ещё.
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("выходим");
                break;
            }
            _ = tick.tick() => {
                let n = srv.users().await?.len();
                println!("[тик] онлайн: {}", n);
            }
            ev = events.recv() => match ev {
                None => {
                    println!("поток закрылся");
                    break;
                }
                Some(Event::UserConnected(u)) => {
                    println!("+ {} (канал {})", u.name, u.channel);
                }
                Some(Event::UserDisconnected(u)) => {
                    println!("- {}", u.name);
                }
                Some(Event::UserTextMessage { user, message }) => {
                    println!("<{}> {}", user.name, message.text);
                }
                Some(Event::ChannelCreated(c)) => println!("# создан {}", c.name),
                Some(Event::ChannelRemoved(c)) => println!("# удалён {}", c.name),
                Some(Event::ChannelStateChanged(c)) => println!("# изменён {}", c.name),
                Some(Event::UserStateChanged(u)) => {
                    println!("~ {} (канал {}, mute={})", u.name, u.channel, u.mute);
                }
                Some(Event::Reattached) => {
                    // Виртуальный сервер перезапустился, подписку фасад вернул.
                    // Все закэшированные SessionId после этого — мусор.
                    println!("! переподписались после перезапуска, состояние надо перечитать");
                }
                Some(Event::Lagged { dropped }) => {
                    // Потери видны, а не молча съедены.
                    eprintln!("! пропущено событий: {}", dropped);
                }
                Some(Event::Error(msg)) => eprintln!("! ошибка: {}", msg),
                Some(other) => println!("? {:?}", other),
            }
        }
    }

    client.shutdown().await;
    Ok(())
}

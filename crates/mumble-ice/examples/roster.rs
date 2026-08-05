//! Показывает состав сервера и рассылает сообщение.
//!
//! ```sh
//! cargo run -p mumble-ice --example roster
//! cargo run -p mumble-ice --example roster -- ssl://murmur.example.com:6502
//! ```
//!
//! Секрет — через `MUMBLE_ICE_SECRET`.

use mumble_ice::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> mumble_ice::Result<()> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("127.0.0.1:6502"));

    let mut builder = MurmurClient::builder().endpoint(endpoint.as_str())?;
    if let Ok(secret) = std::env::var("MUMBLE_ICE_SECRET") {
        builder = builder.secret(secret);
    }
    let client = builder.connect().await?;

    println!("Murmur {}", client.version().await?);
    println!("uptime {:?}", client.uptime().await?);

    let srv = client.only_server().await?;
    println!("виртуальный сервер {} запущен: {}", srv.id(), srv.is_running().await?);

    // Дерево каналов с пользователями — один вызов, настоящий рекурсивный тип.
    let tree = srv.tree().await?;
    for (depth, node) in tree.walk() {
        println!(
            "{:indent$}# {} ({})",
            "",
            node.channel.name,
            node.channel.id,
            indent = depth * 2
        );
        for u in &node.users {
            // Ни одного сентинела: id — Option, online — Duration, address — IpAddr.
            println!(
                "{:indent$}- {} [{}] онлайн {:?}{}{}",
                "",
                u.name,
                match u.id {
                    Some(id) => format!("зарегистрирован {}", id),
                    None => String::from("анонимный"),
                },
                u.online,
                if u.mute { ", заглушен" } else { "" },
                match u.address {
                    Some(a) => format!(", {}", a),
                    None => String::new(),
                },
                indent = depth * 2 + 2
            );
        }
    }

    let registered = srv.registered_users("").await?;
    println!("зарегистрированных пользователей: {}", registered.len());

    // Права — bitflags, а не 0x01 руками.
    if let Some(first) = srv.users().await?.first() {
        let perms = srv
            .effective_permissions(first.session, ChannelId::ROOT)
            .await?;
        println!(
            "{} на корне: speak={} kick={}",
            first.name,
            perms.contains(Permission::SPEAK),
            perms.contains(Permission::KICK)
        );
    }

    match srv.broadcast("привет от mumble-ice").await {
        Ok(()) => println!("сообщение отправлено"),
        // Ошибки различимы по вариантам, а не по тексту.
        Err(e) if e.is_transient() => eprintln!("временная ошибка, можно повторить: {}", e),
        Err(e) => eprintln!("не отправилось: {}", e),
    }

    Ok(())
}

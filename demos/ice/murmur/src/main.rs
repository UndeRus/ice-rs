//! Example Ice client for Murmur (Mumble server). Requires a running murmur with Ice enabled.
//!
//! Usage:
//!   mumble_client [proxy] [secret]
//!
//! Default proxy: `Meta:tcp -h 127.0.0.1 -p 6502` (see murmur.ini icesecretread / port).

use mumble_ice_demo::gen::mumble_server::{Meta, MetaPrx};
use ice_rs::communicator::Communicator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = std::env::args().skip(1);
    let endpoint = args
        .next()
        .unwrap_or_else(|| "Meta:tcp -h 127.0.0.1 -p 6502".to_string());
    let secret = args.next().unwrap_or_default();

    let mut comm = Communicator::new().await?;
    let proxy = comm.string_to_proxy(&endpoint).await?;
    let mut meta = MetaPrx::unchecked_cast(proxy).await?;

    let ctx = if secret.is_empty() {
        None
    } else {
        let mut m = std::collections::HashMap::new();
        m.insert(String::from("secret"), secret);
        Some(m)
    };

    let mut major = 0i32;
    let mut minor = 0i32;
    let mut patch = 0i32;
    let mut text = String::new();
    meta.get_version(&mut major, &mut minor, &mut patch, &mut text, ctx.clone())
        .await?;

    println!("Murmur Meta.getVersion: {}.{}.{} ({})", major, minor, patch, text);

    match meta.get_uptime(ctx).await {
        Ok(s) => println!("Meta.getUptime: {} s", s),
        Err(e) => println!("Meta.getUptime: {}", e),
    }

    Ok(())
}

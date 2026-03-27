use ice_rs::slice::parser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let slice = Path::new(&dir).join("MumbleServer.ice");
    let include = Path::new(&dir).join("include");
    println!("cargo:rerun-if-changed={}", slice.display());
    println!(
        "cargo:rerun-if-changed={}",
        include.join("Ice/SliceChecksumDict.ice").display()
    );
    println!("cargo:rerun-if-changed=build.rs");
    let root = parser::parse_ice_files(
        &vec![slice.to_str().unwrap().to_string()],
        include.to_str().unwrap(),
    )?;
    let out = Path::new(&dir).join("src/gen");
    root.generate(&out, "")?;
    Ok(())
}

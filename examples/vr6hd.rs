//! Example: Telnet client for Roland VR-6HD
//!
//! Usage: cargo run --example vr6hd -- 192.168.0.1 [port]

use roland_rs::TelnetClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <ip_address> [port]", args[0]);
        eprintln!("Example: {} 192.168.0.1", args[0]);
        std::process::exit(1);
    }

    let host = &args[1];
    let port = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(23);

    println!("Connecting to {host}:{port}...");
    let mut client = TelnetClient::connect(host, port)?;
    println!("Connected!");

    match client.get_version() {
        Ok((product, version)) => {
            println!("Product: {product}");
            println!("Version: {version}");
        }
        Err(e) => eprintln!("Error getting version: {e}"),
    }

    match client.read_parameter("000000", 1) {
        Ok(value) => println!("Value at 000000: 0x{value:02X} ({value})"),
        Err(e) => eprintln!("Error reading parameter: {e}"),
    }

    Ok(())
}

//! Example: Telnet client for Roland V-160HD
//!
//! Usage: cargo run --example v160hd -- 192.168.0.1 0000

use roland_rs::devices::v160hd::{self, VideoSource};
use roland_rs::TelnetClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <ip_address> [password]", args[0]);
        eprintln!("Example: {} 192.168.0.1 0000", args[0]);
        std::process::exit(1);
    }

    let host = &args[1];
    let password = args.get(2).map(String::as_str).unwrap_or("0000");

    println!("Connecting to {} port {}...", host, v160hd::TELNET_PORT);
    let mut client = TelnetClient::connect_v160hd(host, password)?;
    println!("Authenticated.");

    match client.get_version() {
        Ok((product, version)) => {
            println!("Product: {product}");
            println!("Version: {version}");
        }
        Err(e) => eprintln!("Error getting version: {e}"),
    }

    let hdmi1 = VideoSource::hdmi(1).map_err(|e| e.to_string())?;
    client.send_command(&v160hd::select_pgm(hdmi1))?;
    println!("Selected HDMI 1 on PGM");

    Ok(())
}

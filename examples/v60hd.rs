//! Example: LAN client for Roland V-60HD
//!
//! Usage:
//!   cargo run --example v60hd -- 192.168.2.254
//!   V60HD_HOST=192.168.2.254 cargo run --example v60hd
//!
//! CUT is skipped unless `V60HD_CUT=1` or `--cut` is passed.

use roland_rs::devices::v60hd::{self, Channel};
use roland_rs::V60HdClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let host = args
        .get(1)
        .cloned()
        .or_else(|| std::env::var("V60HD_HOST").ok());
    let Some(host) = host else {
        eprintln!("Usage: {} <ip_address>", args[0]);
        eprintln!("Example: {} 192.168.2.254", args[0]);
        eprintln!("CUT: V60HD_CUT=1 {} 192.168.2.254", args[0]);
        std::process::exit(1);
    };

    let allow_cut = std::env::var("V60HD_CUT").ok().as_deref() == Some("1")
        || args.iter().any(|a| a == "--cut");

    println!(
        "Connecting to {host} port {} (no password, STX+ACK)...",
        v60hd::TELNET_PORT
    );
    println!("Close V-60HD RCS / other Telnet sessions first (single connection).");

    let mut client = V60HdClient::connect(&host)?;
    println!("Connected.");

    let (product, version) = client.ver()?;
    println!("Product: {product}");
    println!("Version: {version}");

    match client.tly() {
        Ok(tally) => println!("Tally: {tally:?}"),
        Err(e) => eprintln!("TLY error: {e}"),
    }

    match client.qpl_all() {
        Ok(panel) => println!("Panel: {panel:?}"),
        Err(e) => eprintln!("QPL error: {e}"),
    }

    client.send(&v60hd::pst(Channel::Sdi1))?;
    println!("Selected SDI 1 on PST");
    client.send(&v60hd::pgm(Channel::Sdi1))?;
    println!("Selected SDI 1 on PGM");

    if allow_cut {
        client.send(&v60hd::cut())?;
        println!("CUT");
    } else {
        println!("Skipping CUT (set V60HD_CUT=1 or pass --cut to enable)");
    }

    Ok(())
}

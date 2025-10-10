use std::env;
use std::fs;
use std::io::{self, Write, BufRead};
use std::path::Path;
/*

run the following with a desired ip address first, and gateway second,
both should be unique in the subnet your local machine is on
rustc setup_wifi.rs
./setup_wifi 2.2.2.2 2.2.2.3

If you haven't already, you also need to set wifi SSID and password in embassy/examples/rp/wifi.rs

*/

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let ip = args.next().expect("Usage: set_ip <ip-address> <gateway-ip>");
    let gateway = args.next().expect("Usage: set_ip <ip-address> <gateway-ip>");

    // 1. Ensure crusty-gui directory exists, then write to .env
    let gui_dir = Path::new("../crusty-gui");
    if !gui_dir.exists() {
        fs::create_dir_all(gui_dir)?;
    }
    let gui_env_path = gui_dir.join(".env");
    fs::write(&gui_env_path, format!("VITE_IP_ADDRESS={}\n", ip))?;
    println!("Updated {}", gui_env_path.display());

    // 2. Update or create wifi.rs
    let wifi_path = Path::new("../embassy/examples/rp/wifi.rs");
    let ip_parts: Vec<&str> = ip.split('.').collect();
    let gw_parts: Vec<&str> = gateway.split('.').collect();
    if ip_parts.len() != 4 || gw_parts.len() != 4 {
        eprintln!("Invalid IP address format.");
        std::process::exit(1);
    }

    if wifi_path.exists() {
        let mut content = fs::read_to_string(&wifi_path)?;
        let mut changed = false;
        // Update IP and gateway
        content = content
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("const IP_ADDRESS: Ipv4Cidr =") {
                    changed = true;
                    format!(
                        "const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new({}, {}, {}, {}), 24); // change to a unique IP address in the same subnet as your local machine",
                        ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]
                    )
                } else if line.trim_start().starts_with("const GATEWAY: Option<Ipv4Address> =") {
                    changed = true;
                    format!(
                        "const GATEWAY: Option<Ipv4Address> = Some(Ipv4Address::new({}, {}, {}, {})); // change to ANOTHER Unique IP address in the same subnet as your local machine",
                        gw_parts[0], gw_parts[1], gw_parts[2], gw_parts[3]
                    )
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Prompt for SSID and password if empty
        let mut new_content = String::new();
        let mut need_write = false;
        for line in content.lines() {
            if line.trim_start().starts_with("const WIFI_NETWORK: &str = \"\"") {
                let ssid = prompt("Enter WiFi SSID: ");
                new_content.push_str(&format!("const WIFI_NETWORK: &str = \"{}\"; // change to your network SSID\n", ssid));
                need_write = true;
            } else if line.trim_start().starts_with("const WIFI_PASSWORD: &str = \"\"") {
                let pwd = prompt("Enter WiFi password: ");
                new_content.push_str(&format!("const WIFI_PASSWORD: &str = \"{}\"; // change to your network password\n", pwd));
                need_write = true;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
        if changed || need_write {
            fs::write(&wifi_path, new_content)?;
            println!("Updated {}", wifi_path.display());
        }
    } else {
        let ssid = prompt("Enter WiFi SSID: ");
        let pwd = prompt("Enter WiFi password: ");
        let template = format!(
"use embassy_net::Ipv4Address;
use embassy_net::Ipv4Cidr;

const WIFI_NETWORK: &str = \"{}\";
const WIFI_PASSWORD: &str = \"{}\";
const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new({}, {}, {}, {}), 24);
const GATEWAY: Option<Ipv4Address> = Some(Ipv4Address::new({}, {}, {}, {}));
",
            ssid, pwd,
            ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3],
            gw_parts[0], gw_parts[1], gw_parts[2], gw_parts[3]
        );
        if let Some(parent) = wifi_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&wifi_path, template)?;
        println!("Created {}", wifi_path.display());
        println!("You can now edit the file at: {}", wifi_path.canonicalize()?.display());
    }

    Ok(())
}
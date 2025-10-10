use std::fs;
use std::io::{self, Write, BufRead};
use std::net::Ipv4Addr;
use std::path::Path;

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn ping(ip: &Ipv4Addr) -> bool {
    // Cross-platform TCP connect probe (no external ping binary)
    use std::net::{SocketAddrV4, TcpStream};
    use std::time::Duration;

    // Common ports to probe; adjust if you prefer different ports.
    let ports = [80, 443, 22, 53];
    let timeout = Duration::from_millis(500);

    for port in ports {
        let sock_v4 = SocketAddrV4::new(*ip, port);
        let sock = std::net::SocketAddr::V4(sock_v4);
        if TcpStream::connect_timeout(&sock, timeout).is_ok() {
            return true;
        }
    }

    // No successful TCP connection -> treat as unreachable
    false
}

fn main() -> io::Result<()> {
    // 1. Find local IPv4 and subnet
    let adapters = ipconfig::get_adapters().expect("Failed to get network adapters");
    // Print info about all adapters for debugging
    // for adapter in &adapters {
    //     println!("Adapter: {:?}", adapter.friendly_name());
    //     println!("  Status: {:?}", adapter.oper_status());
    //     println!("  IPs: {:?}", adapter.ip_addresses());
    //     println!("  Prefixes: {:?}", adapter.prefixes());
    //     println!("  Gateways: {:?}", adapter.gateways());
    // }

    // Prefer adapters with a default gateway
    let mut local_ip = None;
    let mut prefix_len = None;
    for adapter in &adapters {
        // Only consider adapters that have a gateway
        let has_gateway = adapter.gateways().iter().any(|g| matches!(g, std::net::IpAddr::V4(_)));
        if !has_gateway {
            continue;
        }
        for ip in adapter.ip_addresses() {
            if let std::net::IpAddr::V4(ipv4) = ip {
                if ipv4.is_loopback() || ipv4.is_link_local() {
                    continue;
                }
                // Find the best matching prefix for this IP
                let mut best_prefix = None;
                for (net, pfx) in adapter.prefixes() {
                    if let std::net::IpAddr::V4(netv4) = net {
                        if *pfx < 32 {
                            let mask = u32::MAX << (32 - pfx);
                            if u32::from(*ipv4) & mask == u32::from(*netv4) & mask {
                                if best_prefix.map_or(true, |bp| pfx > &bp) {
                                    best_prefix = Some(*pfx);
                                }
                            }
                        }
                    }
                }
                if let Some(pfx) = best_prefix {
                    local_ip = Some(*ipv4);
                    prefix_len = Some(pfx);
                    break;
                }
            }
        }
        if local_ip.is_some() && prefix_len.is_some() { break; }
    }

    // Fallback: if not found, use any adapter (old logic)
    if local_ip.is_none() || prefix_len.is_none() {
        for adapter in &adapters {
            for ip in adapter.ip_addresses() {
                if let std::net::IpAddr::V4(ipv4) = ip {
                    if ipv4.is_loopback() || ipv4.is_link_local() {
                        continue;
                    }
                    let mut best_prefix = None;
                    for (net, pfx) in adapter.prefixes() {
                        if let std::net::IpAddr::V4(netv4) = net {
                            if *pfx < 32 {
                                let mask = u32::MAX << (32 - pfx);
                                if u32::from(*ipv4) & mask == u32::from(*netv4) & mask {
                                    if best_prefix.map_or(true, |bp| pfx > &bp) {
                                        best_prefix = Some(*pfx);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(pfx) = best_prefix {
                        local_ip = Some(*ipv4);
                        prefix_len = Some(pfx);
                        break;
                    }
                }
            }
            if local_ip.is_some() && prefix_len.is_some() { break; }
        }
    }
    let local_ip = local_ip.expect("Could not find a local IPv4 address");
    let prefix = prefix_len.expect("Could not find a subnet prefix length");
    let netmask = Ipv4Addr::from(!0u32 << (32 - prefix));

    println!("Detected local IP: {}", local_ip);
    println!("Detected subnet mask: {} (/{})", netmask, prefix);

    // 2. Calculate subnet range based on actual mask
    let base = u32::from(local_ip) & u32::from(netmask);
    let broadcast = base | !u32::from(netmask);
    let mut unused_ips = Vec::new();

    for i in (base + 1)..broadcast {
        let candidate = Ipv4Addr::from(i);
        if candidate == local_ip { continue; }
        if !ping(&candidate) {
            unused_ips.push(candidate);
            if unused_ips.len() == 2 { break; }
        }
    }
    if unused_ips.len() < 2 {
        eprintln!("Could not find two unused IP addresses in your subnet.");
        std::process::exit(1);
    }
    let device_ip = unused_ips[0];
    let gateway_ip = unused_ips[1];

    println!("Selected device IP: {}", device_ip);
    println!("Selected gateway IP: {}", gateway_ip);

    // 3. Ensure crusty-gui directory exists, then write to .env
    let gui_dir = Path::new("../crusty-gui");
    if !gui_dir.exists() {
        fs::create_dir_all(gui_dir)?;
    }
    let gui_env_path = gui_dir.join(".env");
    fs::write(&gui_env_path, format!("VITE_IP_ADDRESS={}\n", device_ip))?;
    println!("Updated {}", gui_env_path.display());

    // 4. Update or create wifi.rs
    let wifi_path = Path::new("../embassy/examples/rp/wifi.rs");
    let ip_parts = device_ip.octets();
    let gw_parts = gateway_ip.octets();

    if wifi_path.exists() {
        let content = fs::read_to_string(&wifi_path)?;
        let mut changed = false;
        // Update IP and gateway
        let content = content
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("const IP_ADDRESS: Ipv4Cidr =") {
                    changed = true;
                    format!(
                        "const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new({}, {}, {}, {}), {});",
                        ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3], prefix
                    )
                } else if line.trim_start().starts_with("const GATEWAY: Option<Ipv4Address> =") {
                    changed = true;
                    format!(
                        "const GATEWAY: Option<Ipv4Address> = Some(Ipv4Address::new({}, {}, {}, {}));",
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
                new_content.push_str(&format!("const WIFI_NETWORK: &str = \"{}\";\n", ssid));
                need_write = true;
            } else if line.trim_start().starts_with("const WIFI_PASSWORD: &str = \"\"") {
                let pwd = prompt("Enter WiFi password: ");
                new_content.push_str(&format!("const WIFI_PASSWORD: &str = \"{}\";\n", pwd));
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
const IP_ADDRESS: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new({}, {}, {}, {}), {});
const GATEWAY: Option<Ipv4Address> = Some(Ipv4Address::new({}, {}, {}, {}));
",
            ssid, pwd,
            ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3], prefix,
            gw_parts[0], gw_parts[1], gw_parts[2], gw_parts[3]
        );
        if let Some(parent) = wifi_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&wifi_path, template)?;
        println!("Created {}", wifi_path.display());
    }

    Ok(())
}
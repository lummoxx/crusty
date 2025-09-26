### 1. Install Rust 
https://www.rust-lang.org/tools/install

### 2. Install Node.js
https://nodejs.org/en/download

### 3. If using debug probe, install probe-rs
https://probe.rs/docs/getting-started/installation

### 4. Clone this repo

### 5. Make some local changes
- Set Wifi SSID and password (same as your computer is connected to) in crusty.rs
```
const WIFI_NETWORK: &str = ""; // change to your network SSID
const WIFI_PASSWORD: &str = ""; // change to your network password
```
- Choose a unique IP address in the same subnet as your computer and update net_config.address in crusty.rs
- Choose a unique gateway in the same subnet as your computer and update net_config.gateway in crusty.rs
```
    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 5, 1, 8), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(10, 5, 1, 7)),
    });
```
- also update ipAddress in +page.svelte to be the same as net_config.address
`let ipAddress = $state("10.5.1.8");

### 6. Setup dev environment
`cargo install tauri-cli` or `npm install -g @tauri-apps/cli`
`cargo install create-tauri-app --locked`
`npm install --global yarn`



### 7. Setup car
`cd embassy/examples/rp`
#
#### With debug probe

`cargo run --bin crusty --release`
#
#### Without debug probe
```
cargo install elf2uf2-rs
cargo build --bin crusty --release
elf2uf2-rs ./target/thumbv6m-none-eabi/release/crusty
```

this creates `crusty.uf2` file in `/crusty/embassy/examples/rp/target/thumbv6m-none-eabi/release`

1. Hold the BOOTSEL button on your Pico and connect it to your computer via USB.
2. The Pico will appear as a USB drive.
3. Copy the extracted `.uf2` file onto the Pico’s USB drive.
4. The Pico will reboot and run the new firmware.

### 8. Start dev environment and GUI
```
cd crusty-gui/src-tauri
yarn
yarn add --dev vite
yarn tauri dev
```

#### To get msi
`yarn tauri build`

## Freenove car tutorial
hardware-instructions.pdf in root
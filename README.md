[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?repo=lummoxx/crusty)

# Quick Start with GitHub Codespaces

You can start developing immediately in your browser, with all dependencies pre-installed, using GitHub Codespaces:

1. Click the **"Open in GitHub Codespaces"** badge above, or click the green **Code** button, then select the **Codespaces** tab, and click **"Create codespace on codespaces"** (the `+` button).
2. Wait for the Codespace to build (it uses the included `.devcontainer` for setup).
3. Start coding! All Rust, Node.js, Tauri, and system dependencies are ready to go.

**No local setup or paid licenses required.**

### You need to add wifi credentials
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


# Download and Flash Firmware from Codespaces

To build and flash your firmware using Codespaces, follow these steps:

## 1. Build the Firmware

In the Codespaces terminal, run:

```sh
cd embassy/examples/rp
cargo build --bin crusty --release
```

This will create the firmware binary in the target directory.

## 2. Convert the Firmware to UF2 Format

elf2uf2-rs is already installed in the Codespaces container. To generate the `.uf2` file, run:

```sh
elf2uf2-rs target/thumbv6m-none-eabi/release/crusty
```

This will create a `.uf2` file in the same directory.

## 3. Zip the Firmware Artifact for Download

Return to the root of the repository and run:

```sh
cd $CODESPACE_VSCODE_FOLDER  # or 'cd ../../..' if you are still in embassy/examples/rp
chmod +x scripts/zip-firmware.sh
scripts/zip-firmware.sh
```

This will create a zip file in the `firmware-artifacts` folder containing the latest build output (e.g., `.uf2` and/or binary files).

## 2. Download the Firmware Zip

- In the VS Code file explorer (left sidebar), open the `firmware-artifacts` folder.
- Right-click the newest zip file (e.g., `firmware-YYYYMMDD-HHMMSS.zip`) and select **Download**.
- Extract the zip on your local machine.

## 3. Flash the Firmware to the Car

### Option A: With Debug Probe (probe-rs downloaded on your local machine)
1. Connect your Raspberry Pi Pico (or car) to your computer via the debug probe.
2. Open a terminal on your local machine.
3. Navigate to the extracted folder containing the firmware binary.
4. Run:
    ```sh
    probe-rs download ./crusty --chip rp2040
    ```
    (Or use `probe-rs`/`cargo-flash` directly if you prefer.)

### Option B: Without Debug Probe (Drag-and-Drop)
1. Hold the BOOTSEL button on your Pico and connect it to your computer via USB.
2. The Pico will appear as a USB drive.
3. Copy the extracted `.uf2` file onto the Pico’s USB drive.
4. The Pico will reboot and run the new firmware.

---

# Manual Setup (if not using Codespaces)

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
[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?repo=lummoxx/crusty)

# Quick Start with GitHub Codespaces

You can start developing immediately in your browser, with all dependencies pre-installed, using GitHub Codespaces:

1. Click the **"Open in GitHub Codespaces"** badge above, or click the green **Code** button, then select the **Codespaces** tab, and click **"Create codespace on codespaces"** (the `+` button).
2. Wait for the Codespace to build (it uses the included `.devcontainer` for setup).
3. Start coding! All Rust, Node.js, Tauri, and system dependencies are ready to go.

**No local setup or paid licenses required.**

### You need to add wifi credentials

#### Option 1. Using crate setup_wifi to automate finding a suitable ip address
```
cd setup_wifi
cargo run
```

When prompted, enter your WiFi network name (SSID) and WiFi password.  
These will be securely written to your local configuration file and are required for the device to connect to your network.
note: It must be the same wifi as your local machine is running on.

#### Option 2. Using script setup_wifi.rs with specific ip address and gateway
Run the `setup_wifi.rs` script in the project root to set a unique IP address for your device and a unique gateway, both within the same subnet as your computer.

**Example:**  
If your PC has IP address `2.2.2.1` and subnet mask `255.255.255.0`, and no other machine is using `2.2.2.2` or `2.2.2.3`, you can use those as your device IP and gateway.

**How to check:**
- On **Windows**: Open a terminal and run `ipconfig`
- On **Linux/macOS**: Open a terminal and run `ifconfig` or `ip addr`
- To ensure the IPs are unused, run `ping <chosen-ip>` and `ping <chosen-gateway>`. If there is **no reply**, the address is available.

**Set the IP and gateway:**
Linux/macOS
```sh
rustc setup_wifi.rs && ./setup_wifi 2.2.2.2 2.2.2.3
```
Windows
```sh
rustc setup_wifi.rs
./setup_wifi 2.2.2.2 2.2.2.3
```
*(On Windows PowerShell, use `.\setup_wifi.exe 2.2.2.2 2.2.2.3`)*

When prompted, enter your WiFi network name (SSID) and WiFi password.  
These will be securely written to your local configuration file and are required for the device to connect to your network.
note: It must be the same wifi as your local machine is running on.

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

### 5. Configure Local Network Settings
#### Option 1. Using crate setup_wifi to automate finding a suitable ip address
```
cd setup_wifi
cargo run
```

When prompted, enter your WiFi network name (SSID) and WiFi password.  
These will be securely written to your local configuration file and are required for the device to connect to your network.
note: It must be the same wifi as your local machine is running on.

#### Option 2. Using script setup_wifi.rs with specific ip address and gateway
Run the `setup_wifi.rs` script in the project root to set a unique IP address for your device and a unique gateway, both within the same subnet as your computer.

**Example:**  
If your PC has IP address `2.2.2.1` and subnet mask `255.255.255.0`, and no other machine is using `2.2.2.2` or `2.2.2.3`, you can use those as your device IP and gateway.

**How to check:**
- On **Windows**: Open a terminal and run `ipconfig`
- On **Linux/macOS**: Open a terminal and run `ifconfig` or `ip addr`
- To ensure the IPs are unused, run `ping <chosen-ip>` and `ping <chosen-gateway>`. If there is **no reply**, the address is available.

**Set the IP and gateway:**
Linux/macOS
```sh
rustc setup_wifi.rs && ./setup_wifi 2.2.2.2 2.2.2.3
```
Windows
```sh
rustc setup_wifi.rs
./setup_wifi 2.2.2.2 2.2.2.3
```
*(On Windows PowerShell, use `.\setup_wifi.exe 2.2.2.2 2.2.2.3`)*

When prompted, enter your WiFi network name (SSID) and WiFi password.  
These will be securely written to your local configuration file and are required for the device to connect to your network.
note: It must be the same wifi as your local machine is running on.


### 6. Setup dev environment
```
cargo install tauri-cli` or `npm install -g @tauri-apps/cli
cargo install create-tauri-app --locked
npm install --global yarn
```


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


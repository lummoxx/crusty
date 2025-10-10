### 1. Install Rust 
https://www.rust-lang.org/tools/install

### 2. Install Node.js
https://nodejs.org/en/download

### 3. If using debug probe, install probe-rs
https://probe.rs/docs/getting-started/installation

### 4. Clone this repo

### 5. Configure Local Network Settings

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
# Crusty - WiFi Controlled Car with Raspberry Pi Pico W


## Prerequisites, Setup, and Installation
Please follow the steps below to set up your development environment for Crusty Workshop.
These steps can take time, so please complete them before the workshop begins.
Use a computer with admin rights to install the necessary software.
Firewalls or antivirus software may interfere with some installations, so if your company laptop has restrictions, **consider using a personal device.**

### 1. Install Rust 
https://www.rust-lang.org/tools/install

### 2. Install Node.js
https://nodejs.org/en/download

### 3. Install tools for building and flashing firmware to the Pico
We recommend using a debug probe for easier development, but it's optional. Our cars are all equipped with a debug probe.

#### If using debug probe, install probe-rs
https://probe.rs/docs/getting-started/installation

#### If not using debug probe, install elf2uf2-rs
`cargo install elf2uf2-rs `

### 4. Clone this repo


### 6. Install Tauri 
Before running the GUI, ensure you have the required dependencies for Tauri
https://tauri.app/start/prerequisites/chttps://tauri.app/

Install Tauri CLI
`cargo install tauri-cli` or `npm install -g @tauri-apps/cli`


`cargo install create-tauri-app --locked`


`npm install --global yarn`


## Starting with the car

When you have your hardware in front of you, you can proceed to set up the local network settings.

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


### 7. Setup car
After configuring wifi, you can now flash the firmware to the Pico.

`cd embassy/examples/rp`

#### With debug probe

`cargo run --bin crusty --release`
#### Without debug probe
```
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


## Develop
In files:

- embassy/examples/rp/src/bin/crusty.rs 
- embassy/examples/rp/src/car.rs

you can modify the code to change car behavior.

## Embassy Framework
Our project uses the Embassy framework for embedded Rust development.
Refer to the [Embassy documentation](https://embassy.dev/) for guidance on using the framework and its features.


## Troubleshooting
If you encounter issues during setup or development, please open an issue on the GitHub repository for assistance.
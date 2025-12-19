# Crusty - WiFi Controlled Car with Raspberry Pi Pico W


## Prerequisites, Setup, and Installation
Please follow the steps below to set up your development environment for Crusty Workshop.
These steps can take time, so please complete them before the workshop begins.
Use a computer with admin rights to install the necessary software.
Firewalls or antivirus software may interfere with some installations, so if your company laptop has restrictions, **consider using a personal device.**

### Install Rust 
https://www.rust-lang.org/tools/install

### Install tools for building and flashing firmware to the Pico
We recommend using a debug probe for easier development, but it's optional. Our cars are all equipped with a debug probe.

#### If using debug probe, install probe-rs
https://probe.rs/docs/getting-started/installation

#### If not using debug probe, install elf2uf2-rs
`cargo install elf2uf2-rs `

### Clone this repo

## Starting with the car

When you have your hardware in front of you, you can proceed to set up the local network settings.

### Configure Local Network Settings
#### You need to add wifi credentials
- Set Wifi SSID and password (same as your computer is connected to) in crusty.rs
```
const WIFI_NETWORK: &str = ""; // change to your network SSID
const WIFI_PASSWORD: &str = ""; // change to your network password
```
- Choose a unique IP address in the same subnet as your computer and update net_config.address in crusty.rs
- Choose a unique gateway in the same subnet as your computer and update net_config.gateway in crusty.rs


(if your computer's IP is 10.5.185 with subnet mask 255.255.255.0 you can use something like this)
```
    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 5, 1, 8), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(10, 5, 1, 7)),
    });
```
- also update ipAddress in crusty-bin/src/main.rs to be the same as net_config.address
```
// Set your Pico's IP address here:
const PICO_IP: &str = "10.5.1.8:1234";
```

Make sure the IP address and gateway you choose are not already in use on your network.

### Setup car
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

### Start GUI
**from workspace root**

``cargo run --bin crusty-bin``

**or go to the package dir**
```
cd crusty-bin
cargo run
```
This will start a local web server. Open your web browser and navigate to `http://localhost:8000` to access the car control interface.


## Freenove car tutorial
hardware-instructions.pdf in root

## Develop
To modify or extend the functionality of the Crusty car, you can edit the following source files:

- embassy/examples/rp/src/bin/crusty.rs 
- embassy/examples/rp/src/car.rs


## Embassy Framework
Our project uses the Embassy framework for embedded Rust development.
Refer to the [Embassy documentation](https://embassy.dev/) for guidance on using the framework and its features.

## Troubleshooting
If you encounter issues during setup or development, please open an issue on the GitHub repository.

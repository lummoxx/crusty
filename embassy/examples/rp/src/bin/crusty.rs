#![no_std]
#![no_main]

use core::str::from_utf8;

use cyw43::{Control, JoinOptions};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_net::Ipv4Address;
use embassy_net::Ipv4Cidr;
use embassy_net::Stack;
use embassy_net::{tcp::TcpSocket, StackResources};
use embassy_rp::i2c::{Async, I2c};
use embassy_rp::peripherals::{DMA_CH1, I2C0, I2C1, PIN_16, PIO1};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_rp::{i2c, Peri};
use embassy_rp_examples::car::{initialize_car, Car};
use embassy_time::{Duration, Ticker, Timer};
use embedded_io_async::Write;
use heapless::Vec;
use ht16k33_async::HT16K33;
use rand::RngCore;
use shared::CarCommand;
use smart_leds::RGB8;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Define interrupt handlers
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

// Define interrupt handlers
// bind_interrupts!(struct IC2Irqs {
//     I2C0_IRQ => i2c::InterruptHandler<I2C0>;
// });

const WIFI_NETWORK: &str = ""; // change to your network SSID
const WIFI_PASSWORD: &str = ""; // change to your network password

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Initializing Freenove 4WD Car Control");
    let p = embassy_rp::init(Default::default());

    let mut rng = RoscRng;

    let fw = include_bytes!("../../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // let config = Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 0, 2), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(192, 168, 0, 1)),
    });

    // let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //     address: Ipv4Cidr::new(Ipv4Address::new(10, 5, 1, 2), 24),
    //     dns_servers: Vec::new(),
    //     gateway: Some(Ipv4Address::new(10, 5, 1, 1)),
    // });

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(net_device, config, RESOURCES.init(StackResources::new()), seed);

    // Configure PWM for 500Hz, matching the C++ implementation
    let desired_freq_hz = 500;
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 16u8;
    let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

    let mut config = PwmConfig::default();
    config.top = period;
    config.divider = divider.into();

    let pwm_fl = Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_18, p.PIN_19, config.clone());
    let pwm_fr = Pwm::new_output_ab(p.PWM_SLICE4, p.PIN_8, p.PIN_9, config.clone());
    let pwm_rl = Pwm::new_output_ab(p.PWM_SLICE2, p.PIN_20, p.PIN_21, config.clone());
    let pwm_rr = Pwm::new_output_ab(p.PWM_SLICE3, p.PIN_6, p.PIN_7, config.clone());

    let pio = Pio::new(p.PIO1, Irqs);

    let car = initialize_car(pwm_fl, pwm_fr, pwm_rl, pwm_rr);

    let scl = p.PIN_5;
    let sda = p.PIN_4;
    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, i2c::Config::default());
    let mut driver = HT16K33::new(i2c, 0x71);

    driver.setup().await.unwrap();

    unwrap!(spawner.spawn(led_matrix(driver)));
    unwrap!(spawner.spawn(led_task(pio, p.DMA_CH1, p.PIN_16)));
    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(tcp_task(stack, control, car)));
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[embassy_executor::task]
async fn led_matrix(mut driver: HT16K33<I2c<'static, I2C0, Async>>) {
    let mut buffer = [0u8; 2 * 8];

    loop {
        for c in 0..16 {
            for l in 0..8 {
                buffer[c] ^= 1 << l;

                driver.write_whole_display(&buffer).await.unwrap();

                Timer::after_millis(50).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(mut pio: Pio<'static, PIO1>, dma: Peri<'static, DMA_CH1>, pin: Peri<'static, PIN_16>) {
    let program = PioWs2812Program::new(&mut pio.common);
    let mut ws2812 = PioWs2812::new(&mut pio.common, pio.sm0, dma, pin, &program);

    const NUM_LEDS: usize = 8;
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(10));
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
                debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            ws2812.write(&data).await;
            ticker.next().await;
        }
    }
}

// TCP server task that receives commands and controls the car
#[embassy_executor::task]
async fn tcp_task(stack: Stack<'static>, mut control: Control<'static>, mut car: Car<'static>) {
    // Connect to WiFi
    loop {
        match control
            .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    // Show IP address for debugging
    if let Some(config) = stack.config_v4() {
        info!("IP address: {}", config.address);
    }

    // Set up bincode configuration
    let config = bincode::config::standard();

    // TCP server loop
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        // Set LED off while waiting for connection
        control.gpio_set(0, false).await;
        info!("Listening on TCP:1234...");

        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        // Set LED on after connection established
        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        // Connection handling loop
        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            // Try to parse the received data as a CarCommand using bincode
            match bincode::decode_from_slice::<CarCommand, _>(&buf[..n], config) {
                Ok((command, _)) => {
                    // Execute the command on the car
                    match command {
                        CarCommand::Forward(speed) => {
                            info!("Moving forward with speed {}", speed);
                            car.forward(speed).await;
                        }
                        CarCommand::Backward(speed) => {
                            info!("Moving backward with speed {}", speed);
                            car.backward(speed).await;
                        }
                        CarCommand::TurnLeft(speed) => {
                            info!("Turning left with speed {}", speed);
                            car.turn_left(speed).await;
                        }
                        CarCommand::TurnRight(speed) => {
                            info!("Turning right with speed {}", speed);
                            car.turn_right(speed).await;
                        }
                        CarCommand::Stop => {
                            info!("Stopping car");
                            car.stop().await;
                        }
                    }

                    // Send back acknowledgment
                    let ack = "ACK";
                    if let Err(e) = socket.write_all(ack.as_bytes()).await {
                        warn!("write error: {:?}", e);
                        break;
                    }
                }
                Err(_) => {
                    // If parsing fails, try to interpret as plain text (for debugging)
                    // warn!("Failed to parse command: {:?}", e);

                    if let Ok(str_data) = from_utf8(&buf[..n]) {
                        info!("Received raw data (text): {}", str_data);
                    } else {
                        info!("Received raw data (binary): {:?}", &buf[..n]);
                    }

                    // Echo back the data for debugging purposes
                    match socket.write_all(&buf[..n]).await {
                        Ok(()) => {}
                        Err(e) => {
                            warn!("write error: {:?}", e);
                            break;
                        }
                    };
                }
            }
        }

        // When connection is closed, stop the car for safety
        info!("Connection closed, stopping car");
        // car.stop().await;
    }
}

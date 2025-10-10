#![no_std]
#![no_main]

use core::str::from_utf8;

use cyw43::{Control, JoinOptions};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use embassy_rp::pwm::SetDutyCycle;
use defmt::{info, warn};
use embassy_executor::Spawner;
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
use embassy_rp_examples::car::{initialize_car, Car, Direction};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
use embassy_time::{Duration, Ticker, Timer, Instant};
use embedded_io_async::Write;
use heapless::Vec;
use ht16k33_async::HT16K33;
use rand::RngCore;
use shared::CarCommand;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::gpio::{Input, Pull};
include!("../../wifi.rs");

// ...now use WIFI_NETWORK and WIFI_PASSWORD as before...
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

    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: IP_ADDRESS,
        dns_servers: Vec::new(),
        gateway: GATEWAY,
    });

    // Generate random seed
    let seed = rng.next_u64();
    
    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(net_device, net_config, RESOURCES.init(StackResources::new()), seed);

    stack.wait_config_up().await;
    let ip = stack.config_v4().unwrap().address.address();
    defmt::info!("[main] Assigned IP: {}", ip);

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

    static CAR: StaticCell<Mutex<CriticalSectionRawMutex, Car<'static>>> = StaticCell::new();
    let car = CAR.init(Mutex::new(initialize_car(pwm_fl, pwm_fr, pwm_rl, pwm_rr)));

    // led matrix
    let scl = p.PIN_5;
    let sda = p.PIN_4;
    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, i2c::Config::default());
    let mut driver = HT16K33::new(i2c, 0x71);
    driver.setup().await.unwrap();
    unwrap!(spawner.spawn(led_matrix(driver)));
    
    // ultrasonic module
    // let trig = Output::new(p.PIN_4, Level::Low);
    // let echo = Input::new(p.PIN_5, Pull::Up);
    // unwrap!(spawner.spawn(ultrasonic_task(trig, echo, car)));
    
    // line sensor 
    // let left = Input::new(p.PIN_10, Pull::Up);
    // let middle = Input::new(p.PIN_11, Pull::Up);
    // let right = Input::new(p.PIN_12, Pull::Up);
    // unwrap!(spawner.spawn(line_sensor_task(right, left, middle, car)));

    // servo
    // let mut servo_config = PwmConfig::default();
    // servo_config.top = 20_000;; // 20ms period at 1MHz clock
    // servo_config.divider = 125u8.into(); // adjust for your clock
    // let servo_pwm_13 = Pwm::new_output_b(p.PWM_SLICE6, p.PIN_13, servo_config);
    // unwrap!(spawner.spawn(servo_task(servo_pwm_13)));

    // IR remote
    let ir_pin = Input::new(p.PIN_3, Pull::Up);
    unwrap!(spawner.spawn(ir_task(ir_pin, car)));

    unwrap!(spawner.spawn(led_task(pio, p.DMA_CH1, p.PIN_16)));
    unwrap!(spawner.spawn(net_task(runner)));
    unwrap!(spawner.spawn(tcp_task(stack, control, car)));
} 

#[embassy_executor::task]
async fn line_sensor_task(
    right: Input<'static>,
    left: Input<'static>,
    middle: Input<'static>,
    car: &'static Mutex<CriticalSectionRawMutex, Car<'static>>,
) {
    let mut last_state = (false, false, false);
    loop {
        let state = (
            left.is_high(),
            middle.is_high(),
            right.is_high(),
        );
        if state != last_state {
            let mut car = car.lock().await;
            // do something with the car?
            // car.stop().await;
        }
        last_state = state;
        embassy_time::Timer::after_millis(10).await;
    }
}

fn blue_lights(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (0, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, 0, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (0, 0, 0).into()
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

    loop {

        // Each u8 is a column, each bit is a row (top=LSB)
        let stills: [[u8;16];5] = [
            [
                0b00000000, 0b00000000, 
                0b01111100, 0b01111100, 
                0b10000010, 0b10000010, 
                0b10000010, 0b10000010, 
                0b10001110, 0b10001110, 
                0b10001110, 0b10001110, 
                0b01111100, 0b01111100, 
                0b00000000, 0b00000000, 
            ],
            [
                0b00000000, 0b00000000, 
                0b01111100, 0b01111100, 
                0b10000010, 0b10000010, 
                0b10000010, 0b10000010, 
                0b11100010, 0b11100010, 
                0b11100010, 0b11100010, 
                0b01111100, 0b01111100, 
                0b00000000, 0b00000000, 
            ],
            [
                0b00000000, 0b00000000, 
                0b01111100, 0b01111100, 
                0b11100010, 0b11100010, 
                0b11100010, 0b11100010, 
                0b10000010, 0b10000010, 
                0b10000010, 0b10000010, 
                0b01111100, 0b01111100, 
                0b00000000, 0b00000000, 
            ],
            [
                0b00000000, 0b00000000, 
                0b01111100, 0b01111100, 
                0b10001110, 0b10001110, 
                0b10001110, 0b10001110, 
                0b10000010, 0b10000010, 
                0b10000010, 0b10000010, 
                0b01111100, 0b01111100, 
                0b00000000, 0b00000000, 
            ],
            [
                0b00000000, 0b00000000, 
                0b00000000, 0b00100000, 
                0b00100000, 0b00010000, 
                0b00010000, 0b00001000, 
                0b00001000, 0b00010000, 
                0b00010000, 0b00100000, 
                0b00100000, 0b00000000,
                0b00000000, 0b00000000,
            ],
        ];
        
        for still in stills {
            driver.write_whole_display(&still).await.unwrap();
            Timer::after_millis(500).await;
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
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws2812.write(&data).await;
            ticker.next().await;
        }
    }
}

// TCP server task that receives commands and controls the car
#[embassy_executor::task]
async fn tcp_task(stack: Stack<'static>, mut control: Control<'static>, car: &'static Mutex<CriticalSectionRawMutex, Car<'static>>) {
    // Connect to WiFi
    loop {
        info!("Attempting to join WiFi SSID: {}", WIFI_NETWORK);
        match control
            .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => {
                info!("Successfully joined WiFi!");
                break;
            },
            Err(err) => {
                warn!("join failed with status={}", err.status);
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
                    let mut car = car.lock().await;
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
        // Optionally, you could lock and stop the car here as well
    }
}


#[embassy_executor::task]
async fn servo_task(mut pwm_13: Pwm<'static>) {
/* 
    Servos have three wire leads that usually terminate to a male or female 3-pin
    plug. Two leads are for electric power: positive (2-VCC, Red wire), negative (3-GND, Brown wire), and the
    signal line (1-Signal, Orange wire)

    We will use a 50Hz PWM signal with a duty cycle in a certain range to drive the Servo. 
    The lasting time of 0.5ms-2.5ms of PWM single cycle high level corresponds to the 
    servo angle 0 degrees - 180 degree linearly.

    Part of the corresponding values are as follows:
    High level time -     Servo angle
    0.5ms           -     0 degree
    1ms             -     45 degree
    1.5ms           -     0 degree
    2ms             -     45 degree
    2.5ms           -     180 degree

    When you change the servo signal value, the servo will rotate to the designated angle 
*/
    // Helper: map angle (0-180) to pulse width in microseconds (500-2500us)
    fn angle_to_pulse_us(angle: u16) -> u16 {
        500 + ((angle as u32 * 2000) / 180) as u16
    }

    // Set the sweep range (e.g., 60° to 120°)
    let min_angle: i16 = 60;
    let max_angle: i16 = 120;
    let mut angle: i16 = min_angle;
    let mut dir: i16 = 1;
    loop {
        // Sweep from min_angle to max_angle and back
        let pulse = angle_to_pulse_us(angle as u16);
        let duty = pulse as u16;
        pwm_13.set_duty_cycle(duty);
        Timer::after_millis(20).await;
        angle += dir;
        if angle >= max_angle {
            angle = max_angle;
            dir = -1;
        } else if angle <= min_angle {
            angle = min_angle;
            dir = 1;
        }
    }
}



#[embassy_executor::task]
async fn ultrasonic_task(mut trig: Output<'static>, echo: Input<'static>, car: &'static Mutex<CriticalSectionRawMutex, Car<'static>>) {
    /*
    The ultrasonic ranging module uses the principle that ultrasonic waves will be sent back when encounter
    obstacles. We can measure the distance by counting the time interval between sending and receiving of the
    ultrasonic waves, and the time difference is the total time of the ultrasonic wave’s journey from being
    transmitted to being received. Because the speed of sound in air is a constant, about v=340m/s, we can
    calculate the distance between the ultrasonic ranging module and the obstacle: s=vt/2.

    The ultrasonic ranging module integrates both an ultrasonic transmitter and a receiver. The transmitter is used
    to convert electrical signals (electrical energy) into high frequency (beyond human hearing) sound waves
    (mechanical energy) and the function of the receiver is opposite of this.

    Technical specs:
    Working voltage: 5V 
    Working current: 12mA
    Minimum measured distance: 2cm 
    Maximum measured distance: 200cm

    Instructions for use: 
    output a high-level pulse in Trig pin lasting for least 10us, the module begins to transmit
    ultrasonic waves. At the same time, the Echo pin is pulled up. When the module receives the returned
    ultrasonic waves from encountering an obstacle, the Echo pin will be pulled down. The duration of high level
    in the Echo pin is the total time of the ultrasonic wave from transmitting to receiving, s=vt/2.

    */
    loop {
        {
            let car_guard = car.lock().await;
            if car_guard.direction != Direction::Forward {
                // Only run ultrasonic when moving forward
                drop(car_guard);
                Timer::after_millis(200).await;
                continue;
            }
        }

        trig.set_low();
        Timer::after_micros(2).await;
        trig.set_high();
        Timer::after_micros(10).await;
        trig.set_low();

        let start = Instant::now();
        while !echo.is_high() {
            if start.elapsed().as_micros() > 60000 as u64 {
                break;
            }
            Timer::after_micros(10).await; // Yield to executor
        }

        let echo_start = Instant::now();
        while echo.is_high() {
            if echo_start.elapsed().as_micros() > 60000 as u64 {
                break;
            }
            Timer::after_micros(10).await; // Yield to executor
        }
        let ping_time = echo_start.elapsed().as_micros() as f32; // microseconds

        if ping_time > 1.0 {
            let distance = ping_time * 340.0 / 2.0 / 10000.0;

            if distance < 25.1 {
                debug!("distance: {} ", distance);
                let mut car_guard = car.lock().await;
                car_guard.stop().await;
            }
        }

        Timer::after_millis(200).await; // Increase delay between measurements
    }
}


/// Attempts to read a NEC IR code from the given pin.
/// Returns Some(code) if a valid code is received, or None otherwise.
pub async fn read_ir_code(pin: &Input<'_>) -> Option<u32> {
    // Wait for the start of a transmission (falling edge)
    while pin.is_high() {
        Timer::after_micros(50).await;
    }

    // Measure the length of the initial LOW pulse (should be ~9ms for NEC)
    let start = Instant::now();
    while pin.is_low() {
        if start.elapsed().as_micros() > 12000 {
            // Timeout, not a valid start
            return None;
        }
    }
    let low_duration = start.elapsed().as_micros();
    if low_duration < 8000 || low_duration > 10000 {
        // Not a valid NEC start pulse
        return None;
    }

    // Measure the length of the HIGH pulse (should be ~4.5ms for NEC)
    let start = Instant::now();
    while pin.is_high() {
        if start.elapsed().as_micros() > 6000 {
            return None;
        }
    }
    let high_duration = start.elapsed().as_micros();
    if high_duration < 4000 || high_duration > 5000 {
        return None;
    }

    // Read 32 bits
    let mut code: u32 = 0;
    for i in 0..32 {
        // Wait for LOW (bit start)
        let start = Instant::now();
        while pin.is_low() {
            if start.elapsed().as_micros() > 1000 {
                return None;
            }
        }

        // Measure HIGH duration to determine bit value
        let start = Instant::now();
        while pin.is_high() {
            if start.elapsed().as_micros() > 3000 {
                break;
            }
        }
        let bit_high = start.elapsed().as_micros();

        // NEC: ~560us = 0, ~1.7ms = 1
        let bit = if bit_high > 1200 { 1 } else { 0 };
        code |= bit << (31 - i);
    }

    Some(code)
}


#[embassy_executor::task]
async fn ir_task(ir_pin: Input<'static>, car: &'static Mutex<CriticalSectionRawMutex, Car<'static>>) {
    loop {
        // Print pin state for debugging
        if ir_pin.is_low() {
            debug!("[IR] Pin is LOW, possible signal");
        }
        debug!("[IR] Calling read_ir_code...");
        if let Some(ir_code) = read_ir_code(&ir_pin).await {
            info!("Received IR code: {=u32:X}", ir_code);

            let mut car = car.lock().await;
            let speed = car.speed;
            match ir_code {
                0xFF02FD => car.gas().await,   // '+'
                0xFF9867 => car.brake().await,  // '-'
                0xFFE01F => car.brake_left().await, // '<<'
                0xFF906F => car.brake_right().await,// '>>'
                0xFFA857 => car.stop().await,        // play button
                0xFFC23D => car.reverse().await, // back button
                0xFF6897 => car.set_speed((speed - 10).max(0)).await,
                0xFFB04F => car.set_speed((speed + 10).min(100)).await,
                _ => {},    // do nothing for other codes
                // Add more mappings as needed...
            }
        } else {
            debug!("[IR] No code detected");
        }
        Timer::after_millis(10).await;
    }
}
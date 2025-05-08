#![no_std]
#![no_main]
use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm, PwmOutput, SetDutyCycle};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Structure representing a single wheel with two PWM outputs
struct Wheel<'a> {
    m_in1: PwmOutput<'a>,
    m_in2: PwmOutput<'a>,
}

impl<'a> Wheel<'a> {
    async fn forward(&mut self, speed: u8) {
        // Set forward direction
        self.m_in1.set_duty_cycle_percent(speed).unwrap();
        self.m_in2.set_duty_cycle_fully_off().unwrap();
    }

    async fn back(&mut self, speed: u8) {
        // Set backward direction
        self.m_in1.set_duty_cycle_fully_off().unwrap();
        self.m_in2.set_duty_cycle_percent(speed).unwrap();
    }

    async fn stop(&mut self) {
        // Turn off both PWM signals
        self.m_in1.set_duty_cycle_fully_off().unwrap();
        self.m_in2.set_duty_cycle_fully_off().unwrap();
    }
}

/// Structure representing the entire car with four wheels
struct Car<'a> {
    front_left: Wheel<'a>,
    front_right: Wheel<'a>,
    rear_left: Wheel<'a>,
    rear_right: Wheel<'a>,
}

impl<'a> Car<'a> {
    /// Move the car forward
    async fn forward(&mut self, speed: u8) {
        defmt::info!("Moving car forward at {}% speed", speed);
        self.front_left.forward(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.forward(speed).await;
    }

    /// Move the car backward
    async fn backward(&mut self, speed: u8) {
        defmt::info!("Moving car backward at {}% speed", speed);
        self.front_left.back(speed).await;
        self.front_right.back(speed).await;
        self.rear_left.back(speed).await;
        self.rear_right.back(speed).await;
    }

    /// Turn the car left (tank-style turning)
    async fn turn_left(&mut self, speed: u8) {
        defmt::info!("Turning car left at {}% speed", speed);
        self.front_left.back(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.back(speed).await;
        self.rear_right.forward(speed).await;
    }

    /// Turn the car right (tank-style turning)
    async fn turn_right(&mut self, speed: u8) {
        defmt::info!("Turning car right at {}% speed", speed);
        self.front_left.forward(speed).await;
        self.front_right.back(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.back(speed).await;
    }

    /// Stop all wheels
    async fn stop(&mut self) {
        defmt::info!("Stopping car");
        self.front_left.stop().await;
        self.front_right.stop().await;
        self.rear_left.stop().await;
        self.rear_right.stop().await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Initializing Freenove 4WD Car Control");
    let p = embassy_rp::init(Default::default());

    // Configure PWM for 500Hz, matching the C++ implementation
    let desired_freq_hz = 500;
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 16u8;
    let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

    let mut config = Config::default();
    config.top = period;
    config.divider = divider.into();

    // Initialize PWM for all wheels
    // Front Left: PWM_SLICE1 - PIN_18, PIN_19
    let pwm_fl = Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_18, p.PIN_19, config.clone());
    let (fl_a, fl_b) = pwm_fl.split();
    let front_left = Wheel {
        m_in1: fl_a.unwrap(),
        m_in2: fl_b.unwrap(),
    };

    // Front Right: PWM_SLICE2 - PIN_20, PIN_21
    let pwm_fr = Pwm::new_output_ab(p.PWM_SLICE2, p.PIN_20, p.PIN_21, config.clone());
    let (fr_a, fr_b) = pwm_fr.split();
    let front_right = Wheel {
        m_in1: fr_a.unwrap(),
        m_in2: fr_b.unwrap(),
    };

    // Rear Left: PWM_SLICE3 - PIN_22, PIN_23
    let pwm_rl = Pwm::new_output_ab(p.PWM_SLICE3, p.PIN_22, p.PIN_23, config.clone());
    let (rl_a, rl_b) = pwm_rl.split();
    let rear_left = Wheel {
        m_in1: rl_a.unwrap(),
        m_in2: rl_b.unwrap(),
    };

    // Rear Right: PWM_SLICE4 - PIN_24, PIN_25
    let pwm_rr = Pwm::new_output_ab(p.PWM_SLICE4, p.PIN_24, p.PIN_25, config.clone());
    let (rr_a, rr_b) = pwm_rr.split();
    let rear_right = Wheel {
        m_in1: rr_a.unwrap(),
        m_in2: rr_b.unwrap(),
    };

    // Create car controller with all wheels
    let mut car = Car {
        front_left,
        front_right,
        rear_left,
        rear_right,
    };

    // Test sequence with different movement patterns
    defmt::info!("Starting movement test sequence");
    loop {
        // Forward movement test
        car.forward(50).await;
        Timer::after_millis(2000).await;

        // Stop
        car.stop().await;
        Timer::after_millis(1000).await;

        // Backward movement test
        car.backward(50).await;
        Timer::after_millis(2000).await;

        // Stop
        car.stop().await;
        Timer::after_millis(1000).await;

        // Turn left test
        car.turn_left(50).await;
        Timer::after_millis(1500).await;

        // Stop
        car.stop().await;
        Timer::after_millis(1000).await;

        // Turn right test
        car.turn_right(50).await;
        Timer::after_millis(1500).await;

        // Stop
        car.stop().await;
        Timer::after_millis(2000).await;
    }
}

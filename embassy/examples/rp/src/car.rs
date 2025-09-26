/// Direction of car movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Still,
    Forward,
    Backward,
}

use embassy_rp::pwm::{Pwm, PwmOutput, SetDutyCycle};

/// Structure representing a single wheel with two PWM outputs
pub struct Wheel<'a> {
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
pub struct Car<'a> {
    front_left: Wheel<'a>,
    front_right: Wheel<'a>,
    rear_left: Wheel<'a>,
    rear_right: Wheel<'a>,
    pub speed: u8, // 0-100
    pub direction: Direction,
}

impl<'a> Car<'a> {
    /// Set the car speed, clamped to 0-100
    pub async fn set_speed(&mut self, speed: u8) {
        defmt::info!("Changing speed from {}% to {}%", self.speed, speed);
        self.speed = speed;
    }

    /// Move the car forward
    pub async fn forward(&mut self, speed: u8) {
        defmt::info!("Moving car forward at {}% speed", self.speed);
        self.front_left.forward(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.forward(speed).await;
        self.direction = Direction::Forward;
    }

    /// Move the car backward
    pub async fn backward(&mut self, speed: u8) {
        defmt::info!("Moving car backward at {}% speed", speed);
        self.front_left.back(speed).await;
        self.front_right.back(speed).await;
        self.rear_left.back(speed).await;
        self.rear_right.back(speed).await;
        self.direction = Direction::Backward;
    }

    /// Turn the car left 
    pub async fn turn_left(&mut self, speed: u8) {
        defmt::info!("Turning car left at {}% speed", speed);
        self.front_left.stop().await;
        self.rear_left.stop().await;

        if self.direction == Direction::Forward {
            self.front_right.forward(speed).await;
            self.rear_right.forward(speed).await;
        }
        else {
            self.front_right.back(speed).await;
            self.rear_right.back(speed).await;
        }

    }

    /// Turn the car right 
    pub async fn turn_right(&mut self, speed: u8) {
        defmt::info!("Turning car right at {}% speed", speed);
        self.front_right.stop().await;
        self.rear_right.stop().await;

        if self.direction == Direction::Forward {
            self.front_left.forward(speed).await;
            self.rear_left.forward(speed).await;
        }
        else {
            self.front_left.back(speed).await;
            self.rear_left.back(speed).await;
        }
    }

    /// Stop all wheels
    pub async fn stop(&mut self) {
        defmt::info!("Stopping car");
        self.front_left.stop().await;
        self.front_right.stop().await;
        self.rear_left.stop().await;
        self.rear_right.stop().await;
        self.direction = Direction::Still;
        self.set_speed(0);
    }



    /// Speed up 10%
    pub async fn gas(&mut self) {
        let speed = (self.speed + 10).min(100);
        defmt::info!("Moving car forward at {}% speed", speed);
        self.set_speed(speed).await;
        self.front_left.forward(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.forward(speed).await;
        self.direction = Direction::Forward;
    }
    /// Slow down 10 %
    pub async fn brake(&mut self) {
        let speed = (self.speed - 10).max(0);
        defmt::info!("Moving car forward at {}% speed", speed);
        self.set_speed(speed).await;
        self.front_left.forward(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.forward(speed).await;
        self.direction = Direction::Forward;
    }

    pub async fn reverse(&mut self) {
        self.front_left.back(30).await;
        self.front_right.back(30).await;
        self.rear_left.back(30).await;
        self.rear_right.back(30).await;
        self.direction = Direction::Backward;
    }
    /// Turn the car left 
    pub async fn brake_left(&mut self) {
        defmt::info!("Turning car left");
        self.front_left.stop().await;
        self.rear_left.stop().await;

    }

    /// Turn the car right 
    pub async fn brake_right(&mut self) {
        defmt::info!("Turning car right");
        self.front_right.stop().await;
        self.rear_right.stop().await;

    }
}

pub fn initialize_car<'a>(pwm_fl: Pwm<'a>, pwm_fr: Pwm<'a>, pwm_rl: Pwm<'a>, pwm_rr: Pwm<'a>) -> Car<'a> {
    // Create PWM outputs using references to the peripherals

    let (fl_a, fl_b) = pwm_fl.split();
    let front_left = Wheel {
        m_in1: fl_a.unwrap(),
        m_in2: fl_b.unwrap(),
    };

    // Front Right
    let (fr_a, fr_b) = pwm_fr.split();
    let front_right = Wheel {
        m_in1: fr_b.unwrap(),
        m_in2: fr_a.unwrap(),
    };

    // Rear Left
    let (rl_a, rl_b) = pwm_rl.split();
    let rear_left = Wheel {
        m_in1: rl_b.unwrap(),
        m_in2: rl_a.unwrap(),
    };

    // Rear Right
    let (rr_a, rr_b) = pwm_rr.split();
    let rear_right = Wheel {
        m_in1: rr_b.unwrap(),
        m_in2: rr_a.unwrap(),
    };

    // Create car controller with all wheels
    Car {
        front_left,
        front_right,
        rear_left,
        rear_right,
        speed: 50, // default speed
        direction: Direction::Still,
    }
}

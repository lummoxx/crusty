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
}

impl<'a> Car<'a> {
    /// Move the car forward
    pub async fn forward(&mut self, speed: u8) {
        defmt::info!("Moving car forward at {}% speed", speed);
        self.front_left.forward(speed).await;
        self.front_right.forward(speed).await;
        self.rear_left.forward(speed).await;
        self.rear_right.forward(speed).await;
    }

    /// Move the car backward
    pub async fn backward(&mut self, speed: u8) {
        defmt::info!("Moving car backward at {}% speed", speed);
        // TODO

    }

    /// Turn the car left 
    pub async fn turn_left(&mut self, speed: u8) {
        defmt::info!("Turning car left at {}% speed", speed);
        // TODO

    }

    /// Turn the car right 
    pub async fn turn_right(&mut self, speed: u8) {
        defmt::info!("Turning car right at {}% speed", speed);
        // TODO

    }

    /// Stop all wheels
    pub async fn stop(&mut self) {
        defmt::info!("Stopping car");
        self.front_left.stop().await;
        self.front_right.stop().await;
        self.rear_left.stop().await;
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
    }
}

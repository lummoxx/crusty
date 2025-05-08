#![no_std]
#![no_main]

use bincode::{Decode, Encode};

// Define the command enum for controlling the car
#[derive(Encode, Decode, Debug)]
pub enum CarCommand {
    Forward(u8),   // Forward with specified speed (0-100)
    Backward(u8),  // Backward with specified speed (0-100)
    TurnLeft(u8),  // Turn left with specified speed
    TurnRight(u8), // Turn right with specified speed
    Stop,          // Stop all motors
}

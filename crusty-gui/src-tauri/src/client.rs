use bincode::config::Configuration;
use shared::CarCommand;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct CarClient {
    stream: TcpStream,
    config: Configuration,
}

impl CarClient {
    pub async fn connect(host: &str) -> Result<Self, String> {
        // Default connection parameters
        let port = 1234;
        println!("Attempting to connect to {}:{}", host, port);

        // Set a shorter timeout (e.g., 5 seconds)
        let timeout_duration = std::time::Duration::from_secs(1);

        // Attempt to connect to the TCP server with timeout
        let stream = tokio::time::timeout(timeout_duration, TcpStream::connect(&format!("{host}:{port}")))
            .await
            .map_err(|_| format!("Connection timed out after {:?}", timeout_duration))?
            .map_err(|err| format!("Failed to connect {:?}", err))?;

        let config = bincode::config::standard();

        Ok(Self { stream, config })
    }

    // Send a command to the car and read the acknowledgment
    async fn send_command(&mut self, command: CarCommand) -> Result<String, String> {
        // Serialize the command using bincode
        let encoded = bincode::encode_to_vec(&command, self.config)
            .map_err(|err| format!("Failed to encode command: {:?}", err))?;

        // Send the serialized command
        self.stream
            .write_all(&encoded)
            .await
            .map_err(|err| format!("Failed to send command: {:?}", err))?;

        // Read acknowledgment
        let mut buffer = [0u8; 1024];
        let n = self
            .stream
            .read(&mut buffer)
            .await
            .map_err(|err| format!("Failed to read acknowledgment: {:?}", err))?;

        // Convert acknowledgment to string
        let ack = String::from_utf8_lossy(&buffer[..n]).to_string();

        Ok(ack)
    }

    // Command methods
    pub async fn go_forward(&mut self, speed: u8) -> Result<(), String> {
        println!("Sending forward command with speed {}", speed);
        let command = CarCommand::Forward(speed);
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn go_backward(&mut self, speed: u8) -> Result<(), String> {
        println!("Sending backward command with speed {}", speed);
        let command = CarCommand::Backward(speed);
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn turn_left(&mut self, speed: u8) -> Result<(), String> {
        println!("Sending turn left command with speed {}", speed);
        let command = CarCommand::TurnLeft(speed);
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn turn_right(&mut self, speed: u8) -> Result<(), String> {
        println!("Sending turn right command with speed {}", speed);
        let command = CarCommand::TurnRight(speed);
        self.send_command(command).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        println!("Sending stop command");
        let command = CarCommand::Stop;
        self.send_command(command).await?;
        Ok(())
    }
}

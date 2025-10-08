use std::io::{self, BufRead, Read, Write};
use std::net::TcpStream;
use std::str;

fn main() -> io::Result<()> {
    // Default connection parameters
    let host = "192.168.0.177";
    let port = 1234;

    println!("Attempting to connect to {}:{}", host, port);

    // Attempt to connect to the TCP server
    match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(mut stream) => {
            println!("Successfully connected to server");

            // Set up a reader for user input
            let stdin = io::stdin();
            let mut reader = stdin.lock();

            loop {
                println!("Enter message to send (or 'quit' to exit):");

                // Read input from the user
                let mut message = String::new();
                reader.read_line(&mut message)?;

                // Trim whitespace and check if user wants to quit
                let message = message.trim();
                if message.eq_ignore_ascii_case("quit") {
                    println!("Exiting...");
                    break;
                }

                // Send the message to the server
                stream.write_all(message.as_bytes())?;
                println!("Message sent: {}", message);

                // Read response from server
                let mut response = vec![0; 1024];
                match stream.read(&mut response) {
                    Ok(n) => {
                        if n == 0 {
                            println!("Server closed the connection");
                            break;
                        }

                        let response_text =
                            str::from_utf8(&response[0..n]).unwrap_or("Invalid UTF-8");
                        println!("Server response: {}", response_text);
                    }
                    Err(e) => {
                        println!("Failed to receive response: {}", e);
                        break;
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}

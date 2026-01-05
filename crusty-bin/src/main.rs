use axum::{routing::{get, post}, Router, extract::State, http::StatusCode, Json};
use shared::CarCommand;
use bincode;
use tower_http::services::ServeDir;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use serde::Deserialize;

include!("../../wifi.rs");
// use crusty_controller::car::{initialize_car, Car, Direction};

// Placeholder for your hardware control struct implementing embedded-hal traits
struct HardwareController;
// Implement embedded-hal traits here for HardwareController

#[derive(Clone)]
struct AppState {
    hw: Arc<Mutex<HardwareController>>,
}

// Example control command structure
#[derive(Deserialize)]
struct ControlCommand {
    action: String,
    value: Option<i32>,
}


// Control API handler (POST /control)
async fn control_handler(
    State(_state): State<AppState>,
    Json(cmd): Json<ControlCommand>,
) -> StatusCode {
    // Map incoming command to CarCommand
    let car_cmd = match cmd.action.as_str() {
        "forward" => CarCommand::Forward(cmd.value.unwrap_or(50) as u8),
        "backward" => CarCommand::Backward(cmd.value.unwrap_or(50) as u8),
        "left" => CarCommand::TurnLeft(cmd.value.unwrap_or(50) as u8),
        "right" => CarCommand::TurnRight(cmd.value.unwrap_or(50) as u8),
        "stop" => CarCommand::Stop,
        _ => {
            println!("Unknown action: {}", cmd.action);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Encode with bincode
    let encoded = match bincode::encode_to_vec(car_cmd, bincode::config::standard()) {
        Ok(vec) => vec,
        Err(e) => {
            println!("Bincode encode error: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // Send to Pico over TCP
    // Build socket address from included octet constants
    let pico_addr = format!(
        "{}.{}.{}.{}:{}",
        ADDRESS_OCTETS[0],
        ADDRESS_OCTETS[1],
        ADDRESS_OCTETS[2],
        ADDRESS_OCTETS[3],
        PICO_PORT
    );

    match tokio::net::TcpStream::connect(pico_addr).await {
        Ok(mut stream) => {
            if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut stream, &encoded).await {
                println!("TCP write error: {:?}", e);
                return StatusCode::BAD_GATEWAY;
            }
            StatusCode::OK
        }
        Err(e) => {
            println!("TCP connect error: {:?}", e);
            StatusCode::BAD_GATEWAY
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize hardware controller
    let hw = Arc::new(Mutex::new(HardwareController));
    let state = AppState { hw };

    // Build axum app
    let app = Router::new()
        // Serve static files from crate's static dir (works when run from workspace root)
        .nest_service("/", ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .route("/control", post(control_handler))
        .with_state(state);

    // Listen on 0.0.0.0:8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}/", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}
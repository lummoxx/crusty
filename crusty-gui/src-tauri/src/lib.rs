// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use client::CarClient;

mod client;

#[tauri::command]
async fn forward(speed: u8, ip: String) -> Result<(), String> {
    let mut car_client = CarClient::connect(&ip).await?;
    car_client.go_forward(speed).await?;
    Ok(())
}

#[tauri::command]
async fn backward(speed: u8, ip: String) -> Result<(), String> {
    let mut car_client = CarClient::connect(&ip).await?;
    car_client.go_backward(speed).await?;
    Ok(())
}

#[tauri::command]
async fn left(speed: u8, ip: String) -> Result<(), String> {
    let mut car_client = CarClient::connect(&ip).await?;
    car_client.turn_left(speed).await?;
    Ok(())
}

#[tauri::command]
async fn right(speed: u8, ip: String) -> Result<(), String> {
    let mut car_client = CarClient::connect(&ip).await?;
    car_client.turn_right(speed).await?;
    Ok(())
}

#[tauri::command]
async fn stop(ip: String) -> Result<(), String> {
    let mut car_client = CarClient::connect(&ip).await?;
    car_client.stop().await?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            forward, stop, left, right, backward
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

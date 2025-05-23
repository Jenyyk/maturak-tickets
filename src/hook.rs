// This code never being used is a good thing
#![allow(dead_code)]

use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

use dotenv::dotenv;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

async fn send_webhook(message: &str, url_index: u8) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = if url_index == 0 {
        env::var("LOG_HOOK").expect("Missing LOG_HOOK env variable")
    } else {
        env::var("ALERT_HOOK").expect("Missing ALERT_HOOK env variable")
    };

    // Create webhook payload
    let payload = json!({ "content": message });

    // Create async HTTP Client
    let client = Client::new();

    // Send webhook
    let res = client.post(&url).json(&payload).send().await?; // Await the response

    // Return error if failed
    if !res.status().is_success() {
        eprintln!(
            "Failed sending webhook to Discord. Status: {}",
            res.status()
        );
    }

    Ok(())
}

pub async fn send_file_webhook(file_path: &str) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("BACKUP_HOOK").expect("Missing BACKUP_HOOK env variable");

    // Open the file as a stream
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Convert the file name to a string to avoid lifetime issues
    let file_name = Path::new(file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Create the Part from the file bytes
    let file_part = Part::bytes(buffer).file_name(file_name);

    // Create async HTTP Client
    let client = Client::new();

    // Create the form with the file attachment
    let form = Form::new().part("file", file_part);

    // Send the file via webhook
    let res = client
        .post(&url)
        .multipart(form) // Send the form with the file
        .send()
        .await?;

    // Return error if failed
    if !res.status().is_success() {
        eprintln!(
            "Failed sending file via webhook to Discord. Status: {}",
            res.status()
        );
    }

    Ok(())
}

pub async fn log(text: &str) {
    let _ = send_webhook(&format!("-# Log: {}", text), 0_u8).await;
}
pub async fn warn(text: &str) {
    let _ = send_webhook(&format!(":warning: Varování: {}", text), 1_u8).await;
}
pub async fn panic(text: &str) {
    let _ = send_webhook(&format!("## :red_square: POPLACH: {}", text), 2_u8).await;
}

use futures::executor::block_on;
pub fn log_block(text: &str) {
    block_on(log(text));
}
pub fn warn_block(text: &str) {
    block_on(warn(text));
}
pub fn panic_block(text: &str) {
    block_on(panic(text));
}

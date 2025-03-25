// This code never being used is a good thing
#![allow(dead_code)]

use reqwest::Client;
use serde_json::json;
use std::error::Error;

use dotenv::dotenv;
use std::env;

async fn send_webhook(message: &str) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let url = env::var("ALERT_HOOK").expect("Missing ALERT_HOOK env variable");

    // Create webhook payload
    let payload = json!({ "content": message });

    // Create async HTTP Client
    let client = Client::new();

    // Send webhook
    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await?; // Await the response

    // Return error if failed
    if !res.status().is_success() {
        eprintln!("Failed sending webhook to Discord. Status: {}", res.status());
    }

    Ok(())
}

pub async fn panic(text: &str) {
    let _ = send_webhook(&format!(
        "Přátelé, všechno se dosralo, zde máte shrnutí: {}",
        text
    )).await;
}

use futures::executor::block_on;
pub fn panic_block(text: &str) {
    block_on(panic(text));
}

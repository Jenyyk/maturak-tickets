// This code never being used is a good thing
#![allow(dead_code)]

use reqwest::Client;
use reqwest::multipart::{Form, Part};
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

pub async fn log(text: impl Into<String>) {
    let text = text.into();
    let _ = send_webhook(&format!("-# Log:\n{}", &text), 0_u8).await;
}
pub async fn warn(text: impl Into<String>) {
    let text = text.into();
    let _ = send_webhook(&format!(":warning: Varování:\n{}", &text), 1_u8).await;
}
pub async fn panic(text: impl Into<String>) {
    let text = text.into();
    let _ = send_webhook(&format!("## :red_square: POPLACH:\n{}", &text), 2_u8).await;
}

pub fn log_block(text: impl Into<String>) {
    spawn_new_runtime(log(text.into()));
}

pub fn warn_block(text: impl Into<String>) {
    spawn_new_runtime(warn(text.into()));
}

pub fn panic_block(text: impl Into<String>) {
    spawn_new_runtime(panic(text.into()));
}

fn spawn_new_runtime<F>(fut: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    std::thread::spawn(move || {
        // Create a new multi-threaded runtime with I/O + time drivers enabled.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(fut);
    });
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_hook_validity() {
//         log("Test validity webhooků").await;
//         warn("Test validity webhooků").await;
//         panic("Test validity webhooků").await;
//     }
// }

mod database;
mod hook;
mod kbapi;
mod mail;
mod qrcodes;

use crate::database::Database;
use mail::MailClient;

#[tokio::main]
async fn main() {
    let transactions = kbapi::get_transactions();
    if transactions.is_empty() {
        println!();
        println!("No new transactions found, goodbye!");
        return;
    }

    let mut client = loop {
        match MailClient::new().await {
            Ok(client) => break client,
            Err(_) => print!("retrying. "),
        }
    };

    for transaction in &transactions {
        println!();
        println!("Working on client {}", transaction.address);
        println!("{}", Database::len());
        let transaction_hash = generate_hash(&format!(
            "{}{}{}{}",
            transaction.amount,
            transaction.address,
            transaction.date,
            Database::len()
        ));

        // round up a little (better to lose out on 50 crowns than scam people because of bank fees)
        let amount = (transaction.amount + 100) / 400;

        print!("Checking database... ");
        if Database::contains(&transaction_hash.to_string()) {
            println!("found - cancelling");
            continue;
        }
        println!("not found - continuing");

        let _ = client
            .send_formatted_mail(
                &transaction.address,
                amount as u8,
                transaction_hash.to_string(),
            )
            .await;
    }
    hook::log(&format!("Processed {} new transaction/s", transactions.len())).await;

    Database::backup();
}

use std::hash::{DefaultHasher, Hash, Hasher};
pub fn generate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

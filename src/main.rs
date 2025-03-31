mod bankapi;
mod database;
mod hook;
mod mail;
mod qrcodes;

use crate::database::Database;
use mail::MailClient;

#[tokio::main]
async fn main() {
    let transactions = bankapi::get_transactions();
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

    let mut new_transaction_counter = 0;
    for transaction in &transactions {
        println!();
        println!("Working on client {}", transaction.address);
        print!("Checking database... ");
        if Database::contains(&transaction.transaction_id) {
            println!("found - cancelling");
            continue;
        }
        println!("not found - continuing");
        new_transaction_counter += 1;

        let transaction_hash = generate_hash(&format!(
            "{}{}{}{}",
            transaction.amount,
            transaction.address,
            transaction.date,
            Database::len()
        ));

        // round up a little (better to lose out on 50 crowns than scam people because of bank fees)
        let amount = (transaction.amount + 100) / 400;

        let _ = client
            .send_formatted_mail(
                &transaction.address,
                amount as u8,
                transaction_hash.to_string(),
                transaction.transaction_id.to_string(),
                "normal",
            )
            .await;
    }

    println!();
    Database::backup();

    if new_transaction_counter == 0 {
        return;
    }

    hook::log(&format!(
        "Processed {} new transaction/s",
        new_transaction_counter,
    ))
    .await;
    Database::online_backup().await;
}

use std::hash::{DefaultHasher, Hash, Hasher};
pub fn generate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

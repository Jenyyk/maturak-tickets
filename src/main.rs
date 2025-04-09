mod bankapi;
mod database;
mod hook;
mod mail;
mod qrcodes;

use crate::database::Database;
use mail::MailClient;
use futures::stream::{StreamExt, FuturesUnordered};
use tokio::task;

#[tokio::main]
async fn main() {
    let transactions = bankapi::get_transactions();
    if transactions.is_empty() {
        println!();
        println!("No new transactions found, goodbye!");
        return;
    }

    let mut new_transaction_counter = 0;

    let mut tasks = FuturesUnordered::new();
    for transaction in transactions {
        // Spread sending emails across threads
        tasks.push(task::spawn(async move {
            println!();
            println!("Working on client {}", transaction.address);
            print!("Checking database... ");
            if Database::contains(&transaction.transaction_id) {
                println!("found - cancelling");
                return;
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

            let mut client = loop {
                match MailClient::new().await {
                    Ok(client) => break client,
                    Err(_) => print!("retrying. "),
                }
            };

            let result = client
                .send_formatted_mail(
                    &transaction.address,
                    amount as u8,
                    transaction_hash.to_string(),
                    transaction.transaction_id.to_string(),
                    "normal",
                )
                .await;
            if let Err(_) = result {
                hook::panic_block("nějakej mail se neposlal lol idk snad se to nestane, tahle zprava tu musi byt protože bez ni to nekompiluje");
            }
        }));
    }

    while let Some(_) = tasks.next().await {}

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

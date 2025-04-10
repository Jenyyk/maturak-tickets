mod bankapi;
mod database;
mod hook;
mod mail;
mod qrcodes;

use crate::database::Database;
use mail::MailClient;
use rand::Rng;

#[tokio::main]
async fn main() {
    // manual sponsor ticket hadling
    let args_raw: Vec<String> = std::env::args().collect();
    let mut args = args_raw.iter();

    let mut manual_insertion: bool = false;
    let mut man_email: String = "".to_string();
    let mut man_amount: u8 = 0;
    let mut man_type: String = "normal".to_string();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-m" | "--manual" => manual_insertion = true,
            "-e" | "--email" => man_email = args.next().cloned().expect("Empty flag set"),
            "-t" | "--type" => man_type = args.next().cloned().expect("Empty flag set"),
            "-a" | "--amount" => {
                man_amount = args
                    .next()
                    .cloned()
                    .expect("Empty flag set")
                    .parse::<u8>()
                    .unwrap()
            }
            _ => {}
        }
    }

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
    if manual_insertion {
        println!("Handling manual sponsor mail");
        // random salt is acceptable here as manual mails will only be sent out once (i hope lol)
        let transaction_hash = generate_hash(&format!(
            "{}{}",
            &man_email[..=5],
            rand::rng().random::<u16>()
        ));
        if (client
            .send_formatted_mail(
                &man_email.clone(),
                man_amount,
                transaction_hash.to_string(),
                man_email,
                &man_type,
            )
            .await)
            .is_ok()
        {
            println!("Succesfully handled manual insertion.");
            hook::log("Success on manual ticket insertion.").await;
        }
    }
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

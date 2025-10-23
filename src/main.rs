mod bankapi;
mod database;
mod hook;
mod mail;
mod qrcodes;

use crate::database::Database;
use crate::database::HashStruct;
use mail::MailClient;
use rand::Rng;
use rayon::prelude::*;
use std::sync::Mutex;

#[tokio::main]
async fn main() {
    // manual sponsor ticket hadling
    let args_raw: Vec<String> = std::env::args().collect();
    let mut args = args_raw.iter();

    let mut manual_insertion: bool = false;
    let mut man_email: String = String::new();
    let mut man_amount: u8 = 0;
    let mut man_type: String = String::from("normal");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--del-data" => {Database::delete_data();return},
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

    let transactions = bankapi::get_transactions().await;
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
        if Database::warned_yet(&transaction.transaction_id) {
            println!("saved as invalid - cancelling");
            continue;
        }
        println!("not found - continuing");
        new_transaction_counter += 1;

        let transaction_hash = generate_hash(&format!(
            "{}{}{}{}{}",
            transaction.amount,
            transaction.address,
            transaction.date,
            transaction.transaction_id,
            Database::len(),
        ));

        // round up a little (better to lose out on 50 crowns than scam people because of bank fees)
        let amount = (transaction.amount + 100) / 400;

        // Generate vector of QR code slices and return their hashes
        let (hashes, qr_code_vector) =
            generate_qr_vector(&transaction_hash.to_string(), amount as usize, "normal");
        // Generate a HashStruct now to pass to email function and add to database
        let hash_struct: HashStruct = HashStruct {
            address: transaction.address.clone(),
            hashes,
            transaction_hash: transaction_hash.to_string(),
            transaction_id: transaction.transaction_id.clone(),
            manual: false,
            deleted: false,
        };
        // check for dumbasses
        if &transaction.address == "prosím zadejte svůj e-mail" {
            hook::warn(&format!(
                "Hej nějakej trouba nezadal e-mail xdd, detaily transakce: {}",
                &transaction.to_string()
            ))
            .await;
            Database::add_hash_struct(hash_struct);
            continue;
        }

        let mail_result = client
            .send_formatted_mail(
                &hash_struct,
                amount as u8,
                qr_code_vector
                    .iter()
                    .map(|data| data.as_slice())
                    .collect::<Vec<&[u8]>>(),
            )
            .await;
        match mail_result {
            Ok(_) => Database::add_hash_struct(hash_struct),
            Err(why) => {
                hook::panic(&format!(
                    "Mail se neposlal, informace o mailu:\n{}\nChyba: {:?}",
                    hash_struct,
                    why
                )).await;
                Database::add_invalid_transaction(hash_struct.transaction_id);
                println!("Failed to send mail: {why:?}");
            }
        };
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
        let (hashes, qr_code_vector) = generate_qr_vector(
            &transaction_hash.to_string(),
            man_amount as usize,
            &man_type,
        );
        let hash_struct: HashStruct = HashStruct {
            address: man_email,
            hashes,
            transaction_hash: transaction_hash.to_string(),
            transaction_id: "manual transaction".to_string(),
            manual: true,
            deleted: false,
        };

        if client
            .send_formatted_mail(
                &hash_struct,
                man_amount,
                qr_code_vector
                    .iter()
                    .map(|data| data.as_slice())
                    .collect::<Vec<&[u8]>>(),
            )
            .await
            .is_ok()
        {
            hook::log("handled CLI insertion").await;
            Database::add_hash_struct(hash_struct);
        } else {
            Database::add_invalid_transaction(hash_struct.transaction_id);
            hook::panic("manual sponsor mail did not send").await;
        }
        println!("Manual CLI insertion handled");
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

fn generate_qr_vector(
    transaction_hash: &str,
    amount: usize,
    ticket_type: &str,
) -> (Vec<String>, Vec<Vec<u8>>) {
    println!("Generating QR codes... ");
    let hashes: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let qr_codes: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

    let start_ticket_count = Database::get_ticket_count() + 1;

    // Generate in parallel
    (0..amount)
        .collect::<Vec<usize>>()
        .par_iter()
        .for_each(|&i| {
            let ticket_hash = format!("{}{}", transaction_hash, i);
            let qr_code_image =
                qrcodes::generate_qr_code(&ticket_hash, ticket_type, start_ticket_count + i as u32);

            let mut hashes_guard = hashes.lock().unwrap();
            hashes_guard.push(ticket_hash);

            let mut qr_codes_guard = qr_codes.lock().unwrap();
            qr_codes_guard.push(qr_code_image);

            println!("done with {}", i + 1);
        });

    let hashes_result = hashes.lock().unwrap().clone();
    let qr_code_vector = qr_codes.lock().unwrap().clone();

    println!("done");
    (hashes_result, qr_code_vector)
}

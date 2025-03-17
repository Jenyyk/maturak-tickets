mod database;
mod mail;
mod qrcodes;

use crate::database::Database;
use mail::MailClient;

#[tokio::main]
async fn main() {
    let new_mails = vec!["jan.krivsky@maturak26ab.cz"];

    let mut client = MailClient::new().await.unwrap();

    for address in new_mails {
        println!("Working on client {}", address);
        let address_hash = generate_hash(address);

        print!("Checking database... ");
        if Database::contains(&format!("{}0", address_hash)) {
            println!("found - cancelling");
            continue;
        }
        println!("not found - continuing");

        let _ = client
            .send_formatted_mail(address, 3_u8, address_hash.to_string())
            .await;
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};
pub fn generate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

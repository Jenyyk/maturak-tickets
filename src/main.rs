mod mail;
mod qrcodes;

use mail::MailClient;
use tokio;

#[tokio::main]
async fn main() {
    let new_mails = vec!["jan.krivsky@maturak26ab.cz"];

    let mut client = MailClient::new().await.unwrap();

    for address in new_mails {
        let address_hash = generate_hash(address);
        // TODO!
        // if database.exists(format!("{}0", hash)) { continue; }
        let _ = client.send_formatted_mail(
            address,
            3_u8,
            address_hash.to_string()
        ).await;
    }
}

use std::hash::{DefaultHasher, Hasher, Hash};
pub fn generate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

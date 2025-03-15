mod mail;
use mail::MailClient;
use tokio;

#[tokio::main]
async fn main() {
    let mut client = MailClient::new().await.unwrap();
    client.send_formatted_mail(
        vec!["jan.krivsky@maturak26ab.cz"],
        3_u8,
        186_u16
    ).await;
}

mod mail;
use mail::MailClient;
use tokio;

#[tokio::main]
async fn main() {
    let new_mails = vec!["jan.krivsky@maturak26ab.cz"];

    let mut client = MailClient::new().await.unwrap();
    for address in new_mails {
        let _ = client.send_formatted_mail(
            vec![address],
            3_u8,
            186_u16
        ).await;
    }
}

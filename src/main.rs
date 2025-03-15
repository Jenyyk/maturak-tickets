mod mail;
use mail::MailClient;
use tokio;

#[tokio::main]
async fn main() {
    let mut client = MailClient::new().await.unwrap();
    client.send_mail(
        vec!["jan.krivsky@maturak26ab.cz"],
        "Robotický Mail".to_string(),
        "Ahoj Milý Radečku, <br> Tento mail byl poslán Zcela z robotického programu. Už vim co je SMTP server :)".to_string()
    ).await;
}

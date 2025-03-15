mod mail;
use tokio;

#[tokio::main]
async fn main() {
    let _ = mail::send_mail(
        vec!["jan.krivsky@maturak26ab.cz"],
        "Robotický Mail".to_string(),
        "Ahoj Milý Radečku, <br> Tento mail byl poslán Zcela z robotického programu. Už vim co je SMTP server :)".to_string()
    ).await;
    // let _ = mail::send_mail(
    //     "roub@maturak26ab.cz".to_string(),
    //     "Robotický Mail".to_string(),
    //     "Ahoj Milý Radečku, <br> Tento mail byl poslán Zcela z robotického programu. Už vim co je SMTP server :)".to_string()
    // ).await;
}

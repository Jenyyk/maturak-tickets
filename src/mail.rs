use mail_send;
use mail_send::SmtpClientBuilder;
use mail_send::mail_builder::MessageBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;



pub async fn send_mail(reciever_mails: Vec<&str>, subject: String, html: String) -> Result<(), Box<dyn Error>>{
    let message = MessageBuilder::new()
        .from(("Maturitní Lístky", "listky@maturak26ab.cz"))
        .to(reciever_mails)
        .subject(subject)
        .html_body(html);

    dotenv().ok();
    let password = env::var("MAIL_PASSWORD").expect("MISSING MAIL PASSWORD");
    SmtpClientBuilder::new("smtp.seznam.cz", 465)
        .implicit_tls(true)
        .credentials(("listky@maturak26ab.cz", password.as_str()))
        .connect()
        .await
        .unwrap()
        .send(message)
        .await
        .unwrap();

    Ok(())
}

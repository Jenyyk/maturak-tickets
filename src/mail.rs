use mail_send::{SmtpClient, SmtpClientBuilder};
use mail_send::mail_builder::MessageBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream; // Use tokio_rustls instead of tokio_native_tls

pub struct MailClient {
    client: SmtpClient<TlsStream<TcpStream>>, // Ensure correct TLS implementation
}

use rustls::crypto::{CryptoProvider, ring};
fn init_crypto() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        CryptoProvider::install_default(ring::default_provider())
            .expect("Failed to install Rustls CryptoProvider");
    });
}

impl MailClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        init_crypto();

        dotenv().ok();
        let password = env::var("MAIL_PASSWORD").expect("MISSING MAIL PASSWORD");

        let client = SmtpClientBuilder::new("smtp.seznam.cz", 465)
            .implicit_tls(true)
            .credentials(("listky@maturak26ab.cz", password.as_str()))
            .connect()
            .await?;

        Ok(Self { client })
    }

    pub async fn send_mail(&mut self, receiver_mails: Vec<&str>, subject: String, html: String) -> Result<(), Box<dyn Error>> {
        let message = MessageBuilder::new()
            .from(("Maturitní Lístky", "listky@maturak26ab.cz"))
            .to(receiver_mails)
            .subject(subject)
            .html_body(html);

        self.client.send(message).await?;
        Ok(())
    }
}

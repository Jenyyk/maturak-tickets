use dotenv::dotenv;
use mail_send::mail_builder::MessageBuilder;
use mail_send::{SmtpClient, SmtpClientBuilder};
use std::env;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub struct MailClient {
    client: SmtpClient<TlsStream<TcpStream>>,
}

// We need a crpytography client to store the Smtp connection
use rustls::crypto::{ring, CryptoProvider};
fn init_crypto() {
    println!("Initiating cryptography client for SMTP connection");
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        CryptoProvider::install_default(ring::default_provider())
            .expect("Failed to install Rustls CryptoProvider");
    });
}

// Reads the mail content into memory from a .html file
fn read_html_content() -> Result<String, Box<dyn Error>> {
    Ok(std::fs::read_to_string("message.html")?)
}

use crate::database::HashStruct;

impl MailClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        init_crypto();

        dotenv().ok();
        let password = env::var("MAIL_PASSWORD").expect("MISSING MAIL PASSWORD");

        print!("Connecting to SMTP server... ");
        let client = SmtpClientBuilder::new("smtp.seznam.cz", 465)
            .implicit_tls(true)
            .credentials(("listky@maturak26ab.cz", password.as_str()))
            .connect()
            .await?;
        println!("Connected!");

        Ok(Self { client })
    }

    pub async fn send_mail(
        &mut self,
        receiver_mails: Vec<&str>,
        subject: String,
        html: String,
        qr_codes: Vec<&[u8]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut message = MessageBuilder::new()
            .from(("Maturitní Lístky", "listky@maturak26ab.cz"))
            .to(receiver_mails)
            .subject(subject)
            .html_body(html);

        let filenames: Vec<String> = (1..=qr_codes.len())
            .map(|i| format!("qrcode{}.png", i))
            .collect(); // Store filenames to keep them in memory

        for (code, filename) in qr_codes.iter().zip(&filenames) {
            message = message.attachment("image/png", filename, *code);
        }

        self.client.send(message).await?;
        Ok(())
    }

    // Formats e-mail and sends it
    pub async fn send_formatted_mail(
        &mut self,
        mail_details: &HashStruct,
        ticket_amount: u8,
        qr_code_refs: Vec<&[u8]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut html_content = read_html_content().unwrap();

        // Tohle nám přineslo národní obrození prosím pěkně
        let ticket_amount_formatted = match ticket_amount {
            1 => format!("{ticket_amount} lístek"),
            2..=4 => format!("{ticket_amount} lístky"),
            _ => format!("{ticket_amount} lístků"),
        };

        html_content =
            html_content.replace("{ticket_amount}", &ticket_amount_formatted.to_string());

        print!("Sending formatted e-mail to {}... ", &mail_details.address);
        self.send_mail(
            vec![&mail_details.address],
            "Potvrzení lístků na maturitní ples".to_string(),
            html_content,
            qr_code_refs,
        )
        .await?;
        println!("Sent!");
        Ok(())
    }
}

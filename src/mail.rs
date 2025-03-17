use mail_send::{SmtpClient, SmtpClientBuilder};
use mail_send::mail_builder::MessageBuilder;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub struct MailClient {
    client: SmtpClient<TlsStream<TcpStream>>,
}

// We need a crpytography client to store the Smtp connection
use rustls::crypto::{CryptoProvider, ring};
fn init_crypto() {
    println!("Initiating cryptography client");
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

use crate::qrcodes;
use crate::database::Database;
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

    pub async fn send_mail(&mut self, receiver_mails: Vec<&str>, subject: String, html: String, qr_codes: Vec<&[u8]>) -> Result<(), Box<dyn Error>> {
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
    pub async fn send_formatted_mail(&mut self, receiver_mail: &str, ticket_amount: u8, hash: String) -> Result<(), Box<dyn Error>> {
        let mut html_content = read_html_content().unwrap();

        // Tohle nám přineslo národní obrození prosím pěkně
        let ticket_amount_formatted = match ticket_amount {
            1           => format!("{ticket_amount} lístek"),
            2 | 3 | 4   => format!("{ticket_amount} lístky"),
            _           => format!("{ticket_amount} lístků")
        };

        html_content = html_content.replace("{ticket_amount}", &ticket_amount_formatted.to_string());

        println!("Generating QR codes... ");
        let mut qr_codes: Vec<Vec<u8>> = Vec::new();
        for i in 0..ticket_amount {
            let ticket_hash = format!("{}{}", hash, i);
            let qr_code_image = qrcodes::generate_qr_code(&ticket_hash);
            qr_codes.push(qr_code_image);

            print!("{} ", i);
            Database::add_hash(&ticket_hash);
        }
        println!("done");

        // Now create references that live long enough
        let qr_code_refs: Vec<&[u8]> = qr_codes.iter().map(|data| data.as_slice()).collect();


        print!("Sending formatted e-mail to {}... ", receiver_mail);
        self.send_mail(
            vec![receiver_mail],
            "Potvrzení lístků na maturitní ples".to_string(),
            html_content,
            qr_code_refs
        ).await?;
        println!("Sent!");
        Ok(())
    }
}

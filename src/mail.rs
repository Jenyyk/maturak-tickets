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

use crate::database::{Database, HashStruct};
use crate::hook;
use crate::qrcodes;
use rayon::prelude::*;
use std::sync::Mutex;
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
        receiver_mail: &str,
        ticket_amount: u8,
        transaction_hash: String,
        transaction_id: String,
        ticket_type: &str,
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

        println!("Generating QR codes... ");
        let hashes: Mutex<Vec<String>> = Mutex::new(Vec::new());
        let qr_codes: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
        (0_usize..ticket_amount as usize)
            .collect::<Vec<usize>>()
            .par_iter()
            .for_each(|&i| {
                let ticket_hash = format!("{}{}", transaction_hash, i);
                let qr_code_image = qrcodes::generate_qr_code(&ticket_hash, ticket_type);
                {
                    let mut hashes_guard = hashes.lock().unwrap();
                    hashes_guard.push(ticket_hash);
                }

                {
                    let mut qr_codes_guard = qr_codes.lock().unwrap();
                    qr_codes_guard.push(qr_code_image);
                }
                println!("done with {} ", i + 1);
            });
        println!("done");

        let qr_code_refs: Vec<Vec<u8>> = qr_codes.lock().unwrap().iter().cloned().collect();

        print!("Sending formatted e-mail to {}... ", receiver_mail);
        match self
            .send_mail(
                vec![receiver_mail],
                "Potvrzení lístků na maturitní ples".to_string(),
                html_content,
                qr_code_refs
                    .iter()
                    .map(|data| data.as_slice())
                    .collect::<Vec<&[u8]>>(),
            )
            .await
        {
            Ok(()) => {
                println!("Sent!");
                print!("Adding to database... ");
                Database::add_hash_struct(HashStruct {
                    address: receiver_mail.to_string(),
                    hashes: hashes.lock().unwrap().clone(),
                    transaction_hash,
                    transaction_id,
                    manual: false,
                    deleted: false,
                });
                println!("done");
            }
            Err(e) => {
                hook::panic(&format!("e-mail for {} did not send", receiver_mail)).await;
                println!("failed with Error {}, aborting", e)
            }
        };
        Ok(())
    }
}

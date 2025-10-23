use reqwest::Client;

#[derive(Clone, Debug)]
pub struct Transaction {
    pub amount: u32,
    pub address: String,
    pub date: String,
    pub transaction_id: String,
}

use std::fmt;
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nmessage: {}\ndate: {}\namount: {}\nid: {}",
            self.address, self.date, self.amount, self.transaction_id
        )
    }
}
pub async fn get_transactions() -> Vec<Transaction> {
    fetch_data(7).await
}

use dotenv::dotenv;
use std::env;
pub async fn fetch_data(days_back: u32) -> Vec<Transaction> {
    dotenv().ok();
    // for testing purposes
    if env::var("DEBUG").unwrap_or(String::from("false")) == "true" {
        return vec![Transaction {
            amount: 800,
            address: String::from("listky@maturak26ab.cz"),
            date: String::from("co já vim bru"),
            transaction_id: String::from("0"),
        },
        Transaction {
            amount: 400,
            address: String::from("listky@maturak26ab.cz"),
            date: String::from("co já vim bru"),
            transaction_id: String::from("2"),
        }]
    }

    let api_key = env::var("API_KEY").expect("No FIO API key found");

    let request_url = format!(
        "https://fioapi.fio.cz/v1/rest/periods/{}/{}/{}/transactions.json",
        api_key,
        get_days_back(days_back),
        get_today()
    );

    let client = Client::new();
    let resp = client.get(request_url).send().await.unwrap();
    let raw_json = resp.text().await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&raw_json).unwrap();

    let transactions_array = json["accountStatement"]["transactionList"]["transaction"]
        .as_array()
        .unwrap();

    let transactions: Vec<Transaction> = transactions_array
        .iter()
        .map(|tx| Transaction {
            amount: tx["column1"]["value"].as_f64().unwrap_or(0.0) as u32,
            address: tx["column16"]["value"]
                .as_str()
                .unwrap_or_default()
                .to_string().replace(":", "@"),
            date: millis_to_date(tx["column0"]["value"].as_i64().unwrap_or(0)),
            transaction_id: tx["column22"]["value"]
                .as_i64()
                .unwrap_or(0)
                .to_string(),
        })
        .collect();

    transactions
}

use chrono::{Duration, Local, Utc, TimeZone};
fn get_today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
fn get_days_back(days: u32) -> String {
    let today = Local::now();
    let date_back = today - Duration::days(days as i64);
    date_back.format("%Y-%m-%d").to_string()
}
fn millis_to_date(ms: i64) -> String {
    if ms == 0 {
        return "N/A".to_string();
    }
    let dt = Utc.timestamp_millis_opt(ms).unwrap();
    dt.format("%Y-%m-%d").to_string()
}

// goofy ass DEBUG functions

#[derive(Clone)]
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
pub fn get_transactions() -> Vec<Transaction> {
    fetch_data(20)
}

use dotenv::dotenv;
use std::env;
fn fetch_data(days_back: u32) -> Vec<Transaction> {
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("No FIO API key found");

    let _request_url = format!(
        "https://fioapi.fio.cz/v1/rest/periods/{}/{}/{}/transactions.json",
        api_key,
        get_days_back(days_back),
        get_today()
    );

    vec![
        Transaction {
            amount: 400,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            transaction_id: "0".to_string(),
        },
        Transaction {
            amount: 800,
            address: "listky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            transaction_id: "1".to_string(),
        },
        Transaction {
            amount: 3160,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            transaction_id: "2".to_string(),
        },
        Transaction {
            amount: 3160,
            address: "prosím zadejte svůj e-mail".to_string(),
            date: "18.6.".to_string(),
            transaction_id: "ahoj radečku".to_string(),
        },
        // Transaction {
        //     amount: 750,
        //     address: "roub@maturak26ab.cz".to_string(),
        //     date: "19.3.".to_string(),
        // },
    ]
}

use chrono::{Duration, Local};
fn get_today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
fn get_days_back(days: u32) -> String {
    let today = Local::now();
    let date_back = today - Duration::days(days as i64);
    date_back.format("%Y-%m-%d").to_string()
}

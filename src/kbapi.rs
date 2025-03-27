// goofy ass DEBUG functions

#[derive(Clone)]
pub struct Transaction {
    pub amount: u32,
    pub address: String,
    pub date: String,
}

use std::fmt;
#[derive(Debug)]
pub enum FetchError {
    MissingData,
}
impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchError::MissingData => write!(f, "Requested too little transactions"),
        }
    }
}

use crate::database::Database;
use crate::hook;

pub fn get_transactions() -> Vec<Transaction> {
    let mut size: u16 = 20;
    let transactions = loop {
        // jestli je tohle pravda, tak se něco ukrutně dosralo
        if size > 100 {
            hook::panic_block("Překročen limit pro fetchování");
            return Vec::new();
        }
        match Database::trim_old(fetch_data(size)) {
            Ok(data) => break data,
            Err(FetchError::MissingData) => {
                print!("Not enough data - fetching more... ");
                size += 10;
            }
        };
    };
    if size > 20 {
        hook::warn_block(&format!("Nezvykle velké množství transakcí: {}", size));
    }
    transactions
}

fn fetch_data(_size: u16) -> Vec<Transaction> {
    vec![
        Transaction {
            amount: 400,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
        },
        Transaction {
            amount: 800,
            address: "listky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
        },
        Transaction {
            amount: 750,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
        },
        // Transaction {
        //     amount: 750,
        //     address: "roub@maturak26ab.cz".to_string(),
        //     date: "19.3.".to_string(),
        // },
    ]
}

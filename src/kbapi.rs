// goofy ass DEBUG functions

pub struct Transaction {
    pub amount: u32,
    pub address: String,
    pub date: String,
    pub vs: String,
}

pub fn get_transactions() -> Vec<Transaction> {
    vec![
        Transaction {
            amount: 400,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            vs: "42".to_string(),
        },
        Transaction {
            amount: 800,
            address: "listky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            vs: "0".to_string(),
        },
        Transaction {
            amount: 750,
            address: "jan.krivsky@maturak26ab.cz".to_string(),
            date: "19.3.".to_string(),
            vs: "".to_string(),
        },
    ]
}

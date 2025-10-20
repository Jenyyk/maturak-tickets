use chrono::Local;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Write},
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashStruct {
    pub address: String,
    pub hashes: Vec<String>,
    pub transaction_hash: String,
    pub transaction_id: String,

    pub manual: bool,
    pub deleted: bool,
}

pub struct Database {
    ticket_count: u32,
    data: Vec<HashStruct>,
}

use crate::hook;
impl Database {
    pub fn add_hash_struct(data: HashStruct) {
        let mut db = DATABASE.lock().unwrap();
        db.data.push(data.clone());
        Database::append_to_file("data.txt", &data).unwrap();
    }

    pub fn contains(checking_id: &str) -> bool {
        let db = DATABASE.lock().unwrap();
        for datastruct in &db.data {
            if datastruct.transaction_id == checking_id {
                return true;
            }
        }
        false
    }

    pub fn get_ticket_count() -> u32 {
        let db = DATABASE.lock().unwrap();
        db.ticket_count
    }

    pub fn len() -> usize {
        let mut len = 0;
        let file = fs::read_to_string("data.txt").unwrap();
        for _line in file.lines() {
            len += 1;
        }
        len
    }

    fn load_from_file(file_name: &str) -> Vec<HashStruct> {
        // Ensure the file exists
        if File::open(file_name).is_err() {
            if let Ok(mut file) = File::create(file_name) {
                // Write an empty string to ensure it's a valid file
                let _ = file.write_all(b"");
            }
        }

        // Now open the file safely for reading
        let file = File::open(file_name);
        if file.is_err() {
            return Vec::new(); // Return empty if file still can't be opened
        }

        let reader = io::BufReader::new(file.unwrap());
        let mut data_list = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(entry) = serde_json::from_str::<HashStruct>(&line) {
                if entry.manual {
                    continue;
                }
                data_list.push(entry);
            }
        }

        data_list
    }

    fn append_to_file(file_name: &str, data: &HashStruct) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)?;

        let json = serde_json::to_string(data)?;
        writeln!(file, "{}", json)?; // Append each struct as a new JSON line
        Ok(())
    }

    pub fn backup() {
        print!("Creating backup... ");
        let timestamp = Local::now().format("%d-%m-%Y_%H_%M");
        let backup_name = format!("backups/backup_{}.txt", timestamp);

        fs::create_dir_all("backups").unwrap();
        fs::copy("data.txt", &backup_name).unwrap();
        println!("done");
    }

    pub async fn online_backup() {
        println!("Uploading backup to discord");
        let _ = hook::send_file_webhook("./data.txt").await;
    }
}

// Global singleton instance
static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    let data = Database::load_from_file("./data.txt");
    Mutex::new(Database {
        ticket_count: data.len() as u32,
        data,
    })
});

impl Display for HashStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Adresa: {}\nID transakce: {}",
            self.address, self.transaction_id
        )
    }
}

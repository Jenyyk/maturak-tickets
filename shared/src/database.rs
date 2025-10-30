use chrono::Local;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    error::Error,
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    sync::Mutex,
};

/// HashStruct
///
/// fields:
/// * `address`: String
/// * `hashes`: Vec<String>
///   * a Vec oh QR code hashes belonging to this specific transaction
/// * `transaction_hash`: String
/// * `transaction_id`: String
/// * `manual`: bool
/// * `deleted`: bool
/// * `seen`: Vec<usize>
///   * A Vec of indexes to `hashes` to indicate which hashes have been seen in the event so far
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HashStruct {
    pub address: String,
    pub hashes: Vec<String>,
    pub transaction_hash: String,
    pub transaction_id: String,

    pub manual: bool,
    pub deleted: bool,
    pub seen: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    ticket_count: u32,
    warned_ids: Vec<String>,
    data: Vec<HashStruct>,
}

use crate::hook;

impl Database {
    pub fn add_hash_struct(data: HashStruct) {
        let mut db = DATABASE.lock().unwrap();
        db.data.push(data.clone());
        db.ticket_count += data.hashes.len() as u32;
        db.save_to_file("data.txt").unwrap();
    }

    pub fn add_invalid_transaction(id: String) {
        let mut db = DATABASE.lock().unwrap();
        db.warned_ids.push(id);
        db.save_to_file("data.txt").unwrap();
    }

    pub fn warned_yet(id: &String) -> bool {
        let db = DATABASE.lock().unwrap();
        db.warned_ids.contains(id)
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

    pub fn get_by_hash(checking_hash: &str) -> Option<HashStruct> {
        let db = DATABASE.lock().unwrap();
        db.data
            .iter()
            .find(|datastruct| datastruct.hashes.contains(&String::from(checking_hash)))
            .cloned()
    }

    pub fn mark_ticket_seen(ticket_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = DATABASE.lock().unwrap();
        for hash_struct in &mut db.data {
            for (index, hash) in &mut hash_struct.hashes.iter().enumerate() {
                if hash == ticket_hash {
                    hash_struct.seen.push(index);
                    db.save_to_file("data.txt").unwrap();
                    return Ok(());
                }
            }
        }
        Err("Ticket not found".into())
    }
    pub fn unmark_ticket_seen(ticket_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = DATABASE.lock().unwrap();
        for hash_struct in &mut db.data {
            for (index, hash) in &mut hash_struct.hashes.iter().enumerate() {
                if hash != ticket_hash {
                    continue;
                }
                for (seen_index, seen_element) in
                    hash_struct.seen.clone().into_iter().enumerate()
                {
                    if index != seen_element {
                        continue;
                    }
                    hash_struct.seen.remove(seen_index);
                    db.save_to_file("data.txt").unwrap();
                    return Ok(());
                }
            }
        }
        Err("Ticket not found".into())
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

    fn load_from_file(file_name: &str) -> Database {
        // Ensure the file exists
        if File::open(file_name).is_err()
            && let Ok(mut file) = File::create(file_name)
        {
            // Write an empty string to ensure it's a valid file
            let _ = file.write_all(b"");
        }

        // If we fail opening the database, just fail and warn
        let file = File::open(file_name);
        let file = file.inspect_err(|why| {
            hook::warn_block(&format!("Failed to open database file: {:?}", why));
            panic!("Failed to open database file: {}", why);
        });

        let reader = io::BufReader::new(file.unwrap());

        let mut database: Database = serde_json::from_reader(reader).unwrap_or(Database {
            ticket_count: 0,
            warned_ids: Vec::new(),
            data: Vec::new(),
        });

        // sanity check the ticket count (im lazy)
        let mut ticket_count = 0;
        for hash_struct in &database.data {
            ticket_count += hash_struct.hashes.len() as u32;
        }
        database.ticket_count = ticket_count;

        database
    }

    fn save_to_file(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_name)?;

        let writer = io::BufWriter::new(file);

        serde_json::to_writer(writer, self)?;
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

    #[cfg(debug_assertions)]
    pub fn delete_data() {
        let mut db = DATABASE.lock().unwrap();
        db.data = vec![];
        db.save_to_file("data.txt").unwrap();
    }

    #[cfg(not(debug_assertions))]
    pub fn delete_data() {
        panic!("This function is only available in debug mode");
    }
}

// Global singleton instance
static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    let data: Database = Database::load_from_file("./data.txt");
    Mutex::new(data)
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

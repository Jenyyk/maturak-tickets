use chrono::Local;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    error::Error,
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    sync::{LockResult, Mutex},
};

const DATABASE_PATH: &str = "./data.txt";

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

// Recover database poisoning by reading last valid state from disk
trait DatabaseHeal<T> {
    fn heal(self) -> T;
}
impl<'a> DatabaseHeal<std::sync::MutexGuard<'a, Database>>
    for LockResult<std::sync::MutexGuard<'a, Database>>
{
    fn heal(self) -> std::sync::MutexGuard<'a, Database> {
        match self {
            Ok(db) => db,
            Err(poisoned_lock) => {
                println!("Database was poisoned, recovering!");
                hook::warn_block("Database was poisoned, recovering!");
                DATABASE.clear_poison();
                let mut db = poisoned_lock.into_inner();
                *db = Database::load_from_file(DATABASE_PATH);
                db
            }
        }
    }
}

impl Database {
    pub fn add_hash_struct(data: HashStruct) {
        let mut db = DATABASE.lock().heal();
        db.data.push(data.clone());
        db.ticket_count += data.hashes.len() as u32;
        db.save_to_file(DATABASE_PATH).unwrap();
    }

    pub fn add_invalid_transaction(id: String) {
        let mut db = DATABASE.lock().heal();
        db.warned_ids.push(id);
        db.save_to_file(DATABASE_PATH).unwrap();
    }

    pub fn warned_yet(id: &String) -> bool {
        let db = DATABASE.lock().heal();
        db.warned_ids.contains(id)
    }

    pub fn contains(checking_id: &str) -> bool {
        let db = DATABASE.lock().heal();
        for datastruct in &db.data {
            if datastruct.transaction_id == checking_id {
                return true;
            }
        }
        false
    }

    pub fn get_by_hash(checking_hash: &str) -> Option<HashStruct> {
        let db = DATABASE.lock().heal();
        db.data
            .iter()
            .find(|datastruct| datastruct.hashes.contains(&String::from(checking_hash)))
            .cloned()
    }

    pub fn mark_ticket_seen(ticket_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = DATABASE.lock().heal();
        for hash_struct in &mut db.data {
            for (index, hash) in &mut hash_struct.hashes.iter().enumerate() {
                if hash == ticket_hash {
                    hash_struct.seen.push(index);
                    db.save_to_file(DATABASE_PATH).unwrap();
                    return Ok(());
                }
            }
        }
        Err("Ticket not found".into())
    }
    pub fn unmark_ticket_seen(ticket_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut db = DATABASE.lock().heal();
        for hash_struct in &mut db.data {
            for (index, hash) in &mut hash_struct.hashes.iter().enumerate() {
                if hash != ticket_hash {
                    continue;
                }
                for (seen_index, seen_element) in hash_struct.seen.clone().into_iter().enumerate() {
                    if index != seen_element {
                        continue;
                    }
                    hash_struct.seen.remove(seen_index);
                    db.save_to_file(DATABASE_PATH).unwrap();
                    return Ok(());
                }
            }
        }
        Err("Ticket not found".into())
    }

    pub fn get_ticket_count() -> u32 {
        let db = DATABASE.lock().heal();
        db.ticket_count
    }

    pub fn len() -> usize {
        let mut len = 0;
        let file = match fs::read_to_string(DATABASE_PATH) {
            Ok(file) => file,
            Err(why) => {
                println!("Failed to read database file: {}", why);
                return 20;
            }
        };
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
            hook::warn_block(format!("Failed to open database file: {:?}", why));
            panic!("Failed to open database file: {}", why);
        });

        // safe unwrap here
        let reader = io::BufReader::new(file.unwrap());

        let mut database: Database = serde_json::from_reader(reader).unwrap_or_else(|why| {
            hook::warn_block(format!("Failed to deserialize database {}", why));
            Database {
                ticket_count: 0,
                warned_ids: Vec::new(),
                data: Vec::new(),
            }
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

        fs::create_dir_all("backups").unwrap_or_else(|why| {
            eprintln!("Failed to create backup directory: {why:?}");
        });
        fs::copy(DATABASE_PATH, &backup_name).unwrap_or_else(|why| {
            eprintln!("Failed to copy database to backup: {why:?}");
            0
        });
        println!("done");
    }

    pub async fn online_backup() {
        println!("Uploading backup to discord");

        let _ = hook::send_file_webhook(DATABASE_PATH).await;
    }

    #[cfg(debug_assertions)]
    pub fn delete_data() {
        let mut db = DATABASE.lock().heal();
        db.data = vec![];
        db.save_to_file(DATABASE_PATH).unwrap();
    }

    #[cfg(not(debug_assertions))]
    pub fn delete_data() {
        panic!("This function is only available in debug mode");
    }
}

// Global singleton instance
static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    let data: Database = Database::load_from_file(DATABASE_PATH);
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

#[allow(unused_variables, unreachable_code)]
#[cfg(debug_assertions)]
pub fn debug_panic() {
    let db = DATABASE.lock().unwrap();
    panic!("Debug panic triggered");
    drop(db);
}

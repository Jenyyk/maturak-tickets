use chrono::Local;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Write},
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashStruct {
    pub address: String,
    pub hashes: Vec<String>,
    pub transaction_hash: String,
}

pub struct Database {
    pub data: Vec<HashStruct>,
}

use crate::kbapi;
use crate::kbapi::FetchError;

impl Database {
    pub fn add_hash_struct(data: HashStruct) {
        let mut db = DATABASE.lock().unwrap();
        db.data.push(data.clone());
        Database::append_to_file("data.txt", &data).unwrap();
    }

    pub fn contains(hash: &str) -> bool {
        let db = DATABASE.lock().unwrap();
        for datastruct in &db.data {
            if datastruct.transaction_hash == hash {
                return true;
            }
        }
        false
    }

    pub fn trim_old(new_data: Vec<kbapi::Transaction>) -> Result<Vec<kbapi::Transaction>, FetchError> {
        let data: &Vec<HashStruct> = &DATABASE.lock().unwrap().data;
        // base cases
        if data.is_empty() { return Ok(new_data); }
        if new_data.is_empty() { return Ok(Vec::new()); }
        if data.len() == 1 {
            if data[0].address == new_data[0].address {
                return Ok(new_data);
            }
            return Err(FetchError::MissingData);
        }

        for i in (0..new_data.len()).rev() {
            let mut continuous: bool = true;

            for j in 0..=i {
                if data[data.len() - 1 - j].address != new_data[i - j].address {
                    continuous = false;
                    break;
                }
            }

            // only one element overlaps (cant be sure)
            if continuous && i < 1 {
                return Err(FetchError::MissingData);
            }
            // return only new elements
            if continuous {
                return Ok(new_data[(i+1)..].to_vec());
            }
        }
        Err(FetchError::MissingData)
    }

    pub fn len() -> usize {
        DATABASE.lock().unwrap().data.len()
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
}

// Global singleton instance
static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    Mutex::new(Database {
        data: Database::load_from_file("./data.txt"),
    })
});

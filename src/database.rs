use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, BufRead, Write},
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashStruct {
    pub address: String,
    pub hashes: Vec<String>,
}

pub struct Database {
    pub data: Vec<HashStruct>,
}

impl Database {
    pub fn add_hash_struct(data: HashStruct) {
        let mut db = DATABASE.lock().unwrap();
        db.data.push(data.clone());
        Database::append_to_file("data.txt", &data).unwrap();
    }

    pub fn contains(hash: &str) -> bool {
        let db = DATABASE.lock().unwrap();
        for datastruct in &db.data {
            if datastruct.hashes.contains(&hash.to_string()) {
                return true;
            }
        }
        false
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

        for line in reader.lines() {
            if let Ok(json) = line {
                if let Ok(entry) = serde_json::from_str::<HashStruct>(&json) {
                    data_list.push(entry);
                }
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
}

// Global singleton instance
static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| {
    Mutex::new(Database {
        data: Database::load_from_file("./data.txt"),
    })
});

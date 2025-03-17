use once_cell::sync::Lazy;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    sync::Mutex,
};

pub struct Database {
    pub data: Vec<String>,
}

impl Database {
    pub fn add_hash(hash: &str) {
        let mut db = DATABASE.lock().unwrap();
        db.data.push(hash.to_string());
        Database::append_to_file("data.txt", hash).unwrap()
    }

    pub fn contains(hash: &str) -> bool {
        let db = DATABASE.lock().unwrap();
        db.data.contains(&hash.to_string())
    }

    fn load_from_file(file_name: &str) -> Vec<String> {
        match fs::read_to_string(file_name) {
            Ok(content) => content.lines().map(String::from).collect(),
            Err(e) => { println!("{}", e); Vec::new() },
        }
    }

    fn append_to_file(file_name: &str, hash: &str) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)?;
        writeln!(file, "{}", hash)?;
        Ok(())
    }
}

// Global singleton instnce

static DATABASE: Lazy<Mutex<Database>> = Lazy::new(|| Mutex::new(Database {
    data: Database::load_from_file("data.txt"),
}));

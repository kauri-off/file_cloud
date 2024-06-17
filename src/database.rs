use std::{fs::{self, File, OpenOptions}, io::{self, Read, Write}, path::Path};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    name: String,
    files: Vec<Item>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    name: String,
    size: u64,
    parts: Vec<String>
}

impl Item {
    pub fn new(name: String, size: u64, parts: Vec<String>) -> Self {
        Item {name: name, size: size, parts: parts}
    }

    pub fn parts_iter(&self) -> std::slice::Iter<'_, std::string::String> {
        self.parts.iter()
    }

    pub fn print(&self, current: &str, step: i32) {
        for _i in 0..step {
            eprint!("  ");
        }
        eprint!("- ");

        println!("{}{} | Size: {}", current, self.name, self.size);
    }
}

impl Folder {
    pub fn load(file: &Path) -> io::Result<Self> {
        let mut db = String::new();
        let mut file = File::open(file)?;
        file.read_to_string(&mut db)?;
    
        let db: Folder = serde_json::from_str(&db).unwrap();
    
        Ok(db)
    }
    
    pub fn save(self, file: &Path) -> io::Result<()> {    
        let js = serde_json::to_vec(&self).unwrap();
        if file.exists() {
            fs::remove_file(file)?;
        }
    
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file)?;
        file.write_all(&js)?;
    
        Ok(())
    }

    pub fn add_item(&mut self, item: Item) {
        self.files.push(item);
    }

    pub fn get_file(&self, name: &str) -> Option<Item> {
        for item in self.files.iter() {
            if item.name == name {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn rm_file(&mut self, file: &Item) -> io::Result<()> {
        if let Some(index) = self.files.iter().position(|item| item.name == file.name) {
            self.files.remove(index);
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }
    
        Ok(())
    }    

    // CLI
    pub fn ls(&self) {
        if self.files.len() == 0 {
            return;
        }

        for item in self.files.iter() {
            eprint!("{}  ", item.name);
        }
        println!();
    }

    pub fn ll(&self) {
        for item in self.files.iter() {
            println!("{} | Size: {}", item.name, item.size);
        }
    }

    /// true if created
    pub fn if_not_create(file: &Path) -> bool {
        if file.exists() {
            return false;
        }

        Folder {
            name: String::from(""),
            files: Vec::new()
        }.save(file).unwrap();

        true
    }
}
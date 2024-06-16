use std::{fs::{File, OpenOptions}, io::{self, Read, Write}, path::Path};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    name: String,
    folders: Vec<Folder>,
    files: Vec<Item>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    name: String,
    size: u32,
    parts: Vec<String>
}

impl Item {
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
    
        let mut file = OpenOptions::new()
            .write(true)
            .open(file)?;
        file.write_all(&js)?;
    
        Ok(())
    }

    pub fn print(&self) {
        self.print_tree("", 0);
    }

    pub fn find_file(&self, path: &String) -> Option<Item> {
        let split: Vec<&str> = path.trim().split("/").collect();

        if split.len() < 2 {
            return None;
        }
        let res = self.rec_find_file(&split[1..]);

        res
    }

    pub fn find_folder(&self, path: &String) -> Option<Folder> {
        let split: Vec<&str> = path.trim().split("/").collect();

        if split.len() == 0 {
            return None;
        }
        let res = self.rec_find_folder(&split[1..]);

        res
    }

    fn rec_find_folder(&self, path: &[&str]) -> Option<Folder> {
        if path.len() == 0 {
            return None;
        } else if path.len() == 1 {
            if self.name == path[0] {
                return Some(self.clone());
            }
        }

        for folder in self.folders.iter() {
            if let Some(folder) = folder.rec_find_folder(&path[1..]) {
                return Some(folder);
            }
        }

        None
    }

    fn rec_find_file(&self, path: &[&str]) -> Option<Item> {
        if path.len() > 1 {
            for folder in self.folders.iter() {
                if folder.name == path[0] {
                    return folder.rec_find_file(&path[1..]);
                }
            }
        } else {
            for file in self.files.iter() {
                if file.name == path[0] {
                    return Some(file.clone());
                }
            }
        }

        None
    }

    fn print_tree(&self, previus: &str, step: i32) {
        for _i in 0..step {
            eprint!("  ");
        }

        let current = format!("{}{}/", previus, self.name);
        println!("[{}]", current);

        for file in self.files.iter() {
            file.print(&current, step);
        }

        for folder in self.folders.iter() {
            folder.print_tree(&current, step+1);
        }
    }

    /// true if created
    pub fn if_not_create(file: &Path) -> bool {
        if file.exists() {
            return false;
        }

        Folder {
            name: String::from(""),
            folders: vec![Folder {
                name: String::from("test"),
                folders: vec![],
                files: vec![
                    Item {
                        name: String::from("test.txt"),
                        size: 32,
                        parts: vec![]
                    },
                    Item {
                        name: String::from("test1.txt"),
                        size: 13123,
                        parts: vec![]
                    },
                    Item {
                        name: String::from("test2.txt"),
                        size: 1,
                        parts: vec![]
                    }
                ]
            },
            Folder {
                name: String::from("test2"),
                folders: vec![],
                files: vec![
                    Item {
                        name: String::from("test.txt"),
                        size: 32,
                        parts: vec![]
                    },
                    Item {
                        name: String::from("test1.txt"),
                        size: 13123,
                        parts: vec![]
                    },
                    Item {
                        name: String::from("test2.txt"),
                        size: 1,
                        parts: vec![]
                    }
                ]
            }],
            files: vec![
                Item {
                    name: String::from("test.txt"),
                    size: 32,
                    parts: vec![]
                },
                Item {
                    name: String::from("test.txt"),
                    size: 32,
                    parts: vec![]
                }
                ]
        }.save(file).unwrap();

        true
    }
}
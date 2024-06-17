use std::{io::stdin, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

use console::Term;
use database::{Folder, Item};
use file_util::CryptoInfo;

mod file_util;
mod database;

fn clear() {
    Term::stdout().clear_screen().unwrap();
}

fn print_logo() {
    clear();
    println!("###### ###### ##     #####     ####  ##      ####  ##  ## ##### ");
    println!("##       ##   ##     ##       ##  ## ##     ##  ## ##  ## ##  ##");
    println!("####     ##   ##     ####     ##     ##     ##  ## ##  ## ##  ##");
    println!("##       ##   ##     ##       ##  ## ##     ##  ## ##  ## ##  ##");
    println!("##     ###### ###### #####     ####  ######  ####   ####  ##### ");
    println!();
}

fn main() {
    Folder::if_not_create(Path::new("database.json"));
    let mut db = Folder::load(Path::new("database.json")).unwrap();

    print_logo();

    loop {
        eprint!("[/] # ");

        let mut cmd = String::new();
        stdin().read_line(&mut cmd).unwrap();
        
        let mut split_cmd = cmd.trim().split_whitespace();

        match split_cmd.next() {
            Some("ls") => db.ls(),
            Some("ll") => db.ll(),
            Some("save") => split(&mut db, split_cmd.next(), split_cmd.next()),
            Some("load") => concat(&mut db, split_cmd.next(), split_cmd.next()),
            Some("exit") => break,
            Some(_u) => println!("Unknown command, type help to list commands"),
            None => ()
        }
    }

    db.save(Path::new("database.json")).unwrap();
}

fn split(db: &mut Folder, from: Option<&str>, to: Option<&str>) {
    if from == None || to == None {
        return;
    }
    let from = from.unwrap();
    let to = to.unwrap();

    let path = Path::new(from);
    if !path.exists() {
        println!("File {}, dont exists", from);
        return;
    }
    let size = path.metadata().unwrap().size();

    if let Some(_i) = db.get_file(to) {
        println!("File already exists");
        return;
    }

    let cinfo = CryptoInfo::random();

    let parts = file_util::split_file(path, Path::new("temp_db"), &cinfo);

    match parts {
        Ok(parts) => {
            let item = Item::new(String::from(to), size, parts);
            db.add_item(item);
            eprint!("\n200 ");
        },
        Err(_) => {
            println!("Error");
        },
    }
}

fn concat(db: &mut Folder, from: Option<&str>, to: Option<&str>) {
    if from == None || to == None {
        return;
    }
    let from = from.unwrap();
    let to = to.unwrap();

    let item = match db.get_file(from) {
        Some(item) => item,
        None => {
            println!("File not exists, try ls");
            return;
        }
    };

    let cinfo = CryptoInfo::random();

    let files: Vec<PathBuf> = item.parts_iter()
        .map(|p| Path::new("temp_db").join(p))
        .collect();

    let files_path: Vec<&Path> = files.iter()
        .map(|p| p.as_path())
        .collect();

    let res = file_util::concat_files(files_path, Path::new(to), &cinfo);

    match res {
        Ok(_) => {
            eprint!("\n200 ");
        },
        Err(_) => {
            println!("Error");
        },
    }
}
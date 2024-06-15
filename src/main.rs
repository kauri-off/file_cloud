use std::{io::{self, stdin, Error}, path::Path};

use console::Term;
use database::Folder;

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

fn get_choice() -> io::Result<i32> {
    println!("1 - Split file");
    println!("2 - Load file");
    println!("3 - Exit");

    eprint!(" > ");

    let mut choice = String::new();
    stdin().read_line(&mut choice)?;

    let i = choice.trim().parse::<i32>();

    match i {
        Ok(i) => Ok(i),
        Err(_) => Err(Error::new(io::ErrorKind::InvalidData, "Parsing error")),
    }
}

fn process_split_file(db: &mut Folder) {
    db.print();
}

fn process_save_file(db: &mut Folder) {
    clear();
    db.print();
    println!();
    eprint!("Enter file path: ");

    let mut file = String::new();
    stdin().read_line(&mut file).unwrap();

    let file = db.find_by_path(&file);

    match file {
        Some(file) => {
            println!("Found! {:?}", file);
        },
        None => {
            println!("You are stupid bich");
        }
    };

    let mut file = String::new();
    stdin().read_line(&mut file).unwrap();
}

fn main() {
    Folder::if_not_create(Path::new("database.json"));
    let mut db = Folder::load(Path::new("database.json")).unwrap();

    loop {
        print_logo();
        match get_choice() {
            Ok(1) => process_split_file(&mut db),
            Ok(2) => process_save_file(&mut db),
            Ok(3) => break,
            _e => println!("Are you stupid bich?"),
        };
    }

    db.save(Path::new("database.json")).unwrap();
}
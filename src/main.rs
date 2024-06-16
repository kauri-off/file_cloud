use std::{io::stdin, path::{Path, PathBuf}};

use console::Term;
use database::Folder;
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

    let mut path = vec!["".to_string()];
    print_logo();

    loop {
        eprint!("[/{}] # ", path.join("/"));

        let mut cmd = String::new();
        stdin().read_line(&mut cmd).unwrap();
        
        let mut split_cmd = cmd.trim().split_whitespace();

        match split_cmd.next() {
            Some("ls") => ls_command(&path, &mut db),
            Some("exit") => break,
            Some(_u) => println!("Unknown command, type help to list commands"),
            None => ()
        }
    }

    db.save(Path::new("database.json")).unwrap();
}

fn ls_command(path: &[String], db: &mut Folder) {
    let folder = db.find_folder(&format!("/{}", &path.join("/")));

    dbg!(folder);
}

fn test_split() {
    let cinfo = CryptoInfo::random();

    let files = file_util::split_file(Path::new("test.rar"), Path::new("temp_db"), &cinfo).unwrap();

    let files: Vec<PathBuf> = files.iter()
        .map(|p| Path::new("temp_db").join(p))
        .collect();

    let files_path: Vec<&Path> = files.iter()
        .map(|p| p.as_path())
        .collect();

    file_util::concat_files(files_path, Path::new("test2.rar"), &cinfo).unwrap();
}

fn test_bin2img() {
    file_util::bin2img(Path::new("test.txt")).unwrap();
    file_util::img2bin(Path::new("test.txt")).unwrap();
}
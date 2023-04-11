use rand::seq::IteratorRandom;
use sqlite;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::path::Path;
use std::fs;
use std::env;


pub fn generate_password(length: usize, include_uppercase: bool, include_digits: bool, include_special: bool) -> String {
    let mut password = String::new();

    let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
    let digit_chars = "0123456789";
    let special_chars = "!@#$%^&*()-_=+[]{};:,.<>/?";

    let mut char_set = String::new();

    char_set.push_str(lowercase_chars);

    if include_uppercase { char_set.push_str(lowercase_chars.to_uppercase().as_str()); }
    if include_digits { char_set.push_str(digit_chars); }
    if include_special { char_set.push_str(special_chars); }

    for _ in 0..length {
        let next_char: char = char_set.chars().choose(&mut rand::thread_rng()).expect("Could not generate Password (Error Rand Select).");

        password.push_str(next_char.to_string().as_str());
    }

    password
}


fn get_save_dir() -> String {
    env::home_dir().unwrap().display().to_string() + "/.password-manager/"
}


pub fn create_database_tables() {
    fs::create_dir_all(get_save_dir()).unwrap();
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();

    let query = "CREATE TABLE passwords (id INTEGER PRIMARY KEY AUTOINCREMENT, password TEXT, username TEXT, place TEXT, is_encrypted NUMBER);
CREATE TABLE config (name TEXT, value TEXT);";

    connection.execute(query).unwrap();
}


pub fn save_file_exists() -> bool {
    Path::new(&(get_save_dir().to_owned() + "data.sqlite")).exists()
}


pub fn delete_password(id: &String) {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();

    let query = "DELETE FROM passwords WHERE id = ".to_owned() + id.as_str() + ";";
    
    connection.execute(query).unwrap();
}


pub fn find_password(place: &str) -> Vec<[String; 5]> {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();
    let mut result: Vec<[String; 5]> = Vec::new();

    let query = "SELECT * FROM passwords WHERE place LIKE '".to_string() + place.trim() + "';";

    let mut statement = connection.prepare(query).unwrap();

    while let Ok(sqlite::State::Row) = statement.next() {
        result.push([
            statement.read::<String, _>("password").unwrap(),
            statement.read::<String, _>("username").unwrap(),
            statement.read::<String, _>("place").unwrap(),
            statement.read::<String, _>("id").unwrap(),
            statement.read::<String, _>("is_encrypted").unwrap(),
        ]);
    }
    result
}

pub fn get_all_passwords() -> Vec<[String; 5]> {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();
    let mut result: Vec<[String; 5]> = Vec::new();

    let query = "SELECT * FROM passwords;";

    let mut statement = connection.prepare(query).unwrap();

    while let Ok(sqlite::State::Row) = statement.next() {
        result.push([
            statement.read::<String, _>("password").unwrap(),
            statement.read::<String, _>("username").unwrap(),
            statement.read::<String, _>("place").unwrap(),
            statement.read::<String, _>("id").unwrap(),
            statement.read::<String, _>("is_encrypted").unwrap(),
        ]);
    }
    result
}


pub fn verify_key(key: String) -> String {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();

    let query = "SELECT * FROM config WHERE name = 'verification_str';";

    let mut statement = connection.prepare(query).unwrap();

    while let Ok(sqlite::State::Row) = statement.next() {
        if decrypt(&statement.read::<String, _>("value").unwrap(), key.trim()) == "test decription string used to verify" {
            return key;
        }
    }

    panic!("The inputed encryption key is incorrect.");
}


pub fn save_password(password: &str, username: &str, place: &str, encrypted: bool) {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();

    let write_query = "INSERT INTO passwords (password, username, place, is_encrypted) VALUES ('".to_string() + password.trim() + "', '" + username.trim() + "', '" + place.trim() + "', " + &(encrypted as usize).to_string() + ");";

    match connection.execute(&write_query) {
        Ok(_) => { println!("Save Succesfull."); },
        Err(_) => {
            create_database_tables();
            connection.execute(write_query).expect("Problem saving your data.");
        }
    }
}


pub fn save_key(key: String) {
    let connection = sqlite::open(get_save_dir().to_owned() + "data.sqlite").unwrap();

    let write_query = "INSERT INTO config (name, value) VALUES ('verification_str', '".to_owned() + &encrypt("test decription string used to verify", &key) + "');";
    connection.execute(&write_query).unwrap();
}


pub fn encrypt(string: &str, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);

    mc.encrypt_str_to_base64(string)
}


pub fn decrypt(base64: &str, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);

    mc.decrypt_base64_to_string(base64).unwrap()
}
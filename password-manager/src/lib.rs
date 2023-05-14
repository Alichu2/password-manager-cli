mod password_manager {
    use std::collections::HashMap;
    use std::fs;
    use rand::prelude::IteratorRandom;
    use std::path::PathBuf;
    use dirs::home_dir;
    use sqlite;
    use std::process::exit;
    use magic_crypt::{new_magic_crypt, MagicCryptTrait};
    use super::password_interface::Password;

    trait GetDefault {
        fn get_or_default(&self, key: &str, default: String) -> String;
    }

    impl GetDefault for HashMap<&str, String> {
        fn get_or_default(&self, key: &str, default: String) -> String {
            match self.get(key) {
                Some(result) => result.clone(),
                None => default
            }
        }
    }

    pub struct PasswordManager {
        sample_str_ref: &'static str,
    }

    impl PasswordManager {
        pub fn new() -> Self {
            Self {
                sample_str_ref: "test string to encrypt",
            }
        }

        #[cfg(debug_assertions)]
        fn get_home_path(&self) -> PathBuf {
            match home_dir() {
                None => {
                    println!("Error: Home Directory Path Not Found.");
                    exit(1);
                }
                Some(path) => path
            }
        }

        #[cfg(not(debug_assertions))]
        fn get_home_path(&self) -> PathBuf {
            PathBuf::from("./")
        }

        fn get_save_dir_path(&self) -> PathBuf {
            self.get_home_path().join(".password-manager/")
        }

        fn get_save_file_path(&self) -> PathBuf {
            self.get_save_dir_path().join("data.sqlite")
        }

        fn get_save_file_path_str(&self) -> String {
            self.get_save_file_path().display().to_string()
        }

        pub fn save_file_exists(&self) -> bool {
            self.get_save_file_path().exists()
        }

        fn get_sqlite_connection(&self) -> sqlite::Connection {
            match sqlite::open(&self.get_save_file_path_str()) {
                Ok(connection) => connection,
                Err(_) => {
                    println!("An error occurred when trying to open the save file. This might be due to the fact that the file is not generated. Generate it with `password-manager create`");
                    exit(1);
                }
            }
        }

        fn execute_sql(&self, query: &str) -> Result<(), ()> {
            let connection = self.get_sqlite_connection();

            match connection.execute(query) {
                Ok(_) => Ok(()),
                Err(_) => Err(())
            }
        }

        fn read_sql_data<'a>(&'a self, fields: Vec<&'a str>, search_query: &str) -> Result<Vec<HashMap<&str, String>>, ()> {
            let connection = self.get_sqlite_connection();

            let result = match connection.prepare(search_query) {
                Ok(mut statement) => {
                    let mut result_data: Vec<HashMap<&str, String>> = Vec::new();

                    while let Ok(sqlite::State::Row) = statement.next() {
                        let mut row_data: HashMap<&str, String> = HashMap::new();

                        for field in fields.iter() {
                            let value = statement.read::<String, _>(field.clone()).expect("Error reading values from save file.");
                            row_data.insert(field, value);
                        }

                        result_data.push(row_data);
                    }

                    Ok(result_data)
                },
                Err(_) => Err(())
            };
            result
        }

        pub fn generate(&self, length: usize, include_uppercase: bool, include_digits: bool, include_special: bool) -> String {
            let mut generated_password = String::new();

            let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
            let digit_chars = "0123456789";
            let special_chars = "!@#$%^&*()-_=+[]{};:,.<>/?";

            let mut char_set = String::new();

            char_set.push_str(lowercase_chars);

            if include_uppercase { char_set.push_str(lowercase_chars.to_uppercase().as_str()); }
            if include_digits { char_set.push_str(digit_chars); }
            if include_special { char_set.push_str(special_chars); }

            for _ in 0..length {
                let next_char: char = char_set.chars().choose(&mut rand::thread_rng())
                    .expect("Could not generate password (Error Rand Select). If this issue persists, please create a github issue at https://github.com/Alichu2/password-manager-cli");

                generated_password.push_str(next_char.to_string().as_str());
            }

            generated_password
        }

        pub fn save_new_key(&self, key: String) {
            let encrypted_sample_str = self.encrypt(self.sample_str_ref, &key);

            match self.execute_sql("") {
                Ok(_) => println!("Key saved."),
                Err(_) => println!("An issue occurred while trying to save your key.")
            } // Needs sql command added to write encrypted value
        }

        pub fn save_password(&self, password: &str, username: &str, place: &str, encrypted: bool) -> Result<(), ()> {
            self.execute_sql(&("INSERT INTO passwords (password, username, place, is_encrypted) VALUES ('".to_string() + password.trim() + "', '" + username.trim() + "', '" + place.trim() + "', " + &(encrypted as usize).to_string() + ");"))
        }

        pub fn delete_password(&self, id: &str) -> Result<(), ()> {
            self.execute_sql("")
        }

        pub fn get_passwords(&self, search_query: &str) -> Result<Vec<Password>, ()> {
            match self.read_sql_data::<'static>(vec!["password", "username", "place", "id", "is_encrypted"], search_query) {
                Ok(val) => {
                    let mut unpacked_passwords = Vec::new();

                    for packed_password in val.iter() {
                        unpacked_passwords.push(Password {
                            password: packed_password.get_or_default("password", String::from("none")),
                            place: packed_password.get_or_default("place", String::from("none")),
                            username: packed_password.get_or_default("username", String::from("none")),
                            id: packed_password.get_or_default("id", String::from("0")).parse::<usize>().expect("Error parsing password."),
                            encrypted: packed_password.get_or_default("encrypted", String::from("1")) == String::from("1"),
                        })
                    }

                    Ok(unpacked_passwords)
                },
                Err(_) => Err(())
            }
        }

        pub fn create_save_file(&self) -> Result<(), ()> {
            match fs::create_dir_all(self.get_save_dir_path().display().to_string()) {
                Ok(_) => {
                    match self.execute_sql("CREATE TABLE passwords (id INTEGER PRIMARY KEY AUTOINCREMENT, password TEXT, username TEXT, place TEXT, is_encrypted NUMBER);
CREATE TABLE config (name TEXT, value TEXT);") {
                        Ok(_) => Ok(()),
                        Err(_) => Err(())
                    }
                },
                Err(_) => Err(())
            }
        }

        pub fn verify_key(&self, key: &str) -> bool {
            match self.read_sql_data::<'static>(vec!["value"], "SELECT * FROM config WHERE name = 'verification_str';") {
                Ok(result) => {
                    let value = match result[0].get("value") {
                        None => {
                            println!("No sample string found to compare key to. Try again or report issue.");
                            exit(1);
                        },
                        Some(val) => val
                    };
                    let decrypted_string = match self.decrypt(&value, &key) {
                        Ok(val) => val,
                        Err(_) => String::from(""),
                    };

                    decrypted_string == self.sample_str_ref
                },
                Err(_) => {
                    println!("An error occurred when trying to read from the save file.");
                    exit(1);
                }
            }
        }

        pub fn encrypt(&self, string: &str, key: &str) -> String {
            let mc = new_magic_crypt!(key, 256);

            mc.encrypt_str_to_base64(string)
        }

        pub fn decrypt(&self, base64: &str, key: &str) -> Result<String, ()> {
            let mc = new_magic_crypt!(key, 256);

            match mc.decrypt_base64_to_string(base64) {
                Ok(result) => Ok(result),
                Err(_) => Err(())
            }
        }
    }
}

pub mod password_interface {
    use super::password_manager::PasswordManager;
    use std::process::exit;

    pub struct Password {
        pub password: String,
        pub username: String,
        pub place: String,
        pub encrypted: bool,
        pub id: usize,
    }

    pub enum PasswordDeletionStatus {
        Ok,
        Err,
        Multiple
    }

    pub struct PasswordManagerInterface {
        pw_core: PasswordManager,
    }

    impl PasswordManagerInterface {
        pub fn new() -> Self {
            Self {
                pw_core: PasswordManager::new(),
            }
        }

        fn get_key_if_required(&self, passwords: &Vec<Password>) -> Option<String> {
            for password in passwords.iter() {
                if password.encrypted {
                    return Some(String::from("temp_key"));
                }
            }
            None
        }

        fn print_passwords(&self, passwords: Vec<Password>) {
            let key = match self.get_key_if_required(&passwords) {
                Some(key) => key,
                None => String::new()
            };
            let mut password_count: usize = 0;

            for password in passwords.iter() {
                println!("{}:", password_count);
                println!("  place = {}", &password.place);
                println!("  username = {}", &password.username);
                if password.encrypted {
                    println!("  password = {}", match self.pw_core.decrypt(&key, &password.password) {
                        Ok(val) => val,
                        Err(_) => {
                            println!("Problem decrypting your password. Try again or report an issue.");
                            exit(1);
                        }
                    });
                }
                else {
                    println!("  password = {}", &password.password);
                }

                password_count += 1;
            }
        }

        pub fn load_password(&self, place: &str) {
            let result = self.pw_core.get_passwords(&("SELECT * FROM passwords WHERE place LIKE '%".to_string() + place + "%';"));

            match result {
                Ok(val) => {
                    self.print_passwords(val);
                },
                Err(_) => {
                    println!("An error occurred when loading your passwords. Please try again or report an issue.");
                    exit(1);
                }
            }
        }

        pub fn create_save_file(&self, new_key: &str) {
            if !self.pw_core.save_file_exists() {
                match self.pw_core.create_save_file() {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Error creating save file.");
                        exit(1);
                    }
                }
                self.pw_core.save_new_key(new_key.to_string());
            }
            else {
                println!("Save file and key already exists. Cannot regenerate.");
            }
        }

        pub fn generate_password(&self, save: bool, special_char: bool, upper_case: bool, digits: bool, length: usize, uname: String, place: String, encrypt: bool, key: String) {
            let generated_password = self.pw_core.generate(length, upper_case, digits, special_char);

            if save {
                let saving_password: String;

                if encrypt {
                    saving_password = self.pw_core.encrypt(&generated_password, &key);
                }
                else {
                    saving_password = generated_password.to_string();
                }

                match self.pw_core.save_password(&saving_password, &uname, &place, encrypt) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Error occurred while saving password.");
                        exit(1);
                    }
                }
                println!("saved password:\n  password = {}\n  username = {}\n  place = {}", generated_password, uname, place);
            }
            else {
                println!("Generated password = {}", generated_password);
            }
        }

        pub fn delete_password(&self, place: String, using_id: bool, id: String) -> PasswordDeletionStatus {
            if using_id {

            }
            let passwords = match self.pw_core.get_passwords(&("SELECT * FROM passwords WHERE place LIKE '%".to_string() + &place + "%';")) {
                Ok(val) => val,
                Err(_) => {
                    println!("Error deleting password.");
                    exit(1);
                }
            };

            if passwords.len() > 1 {
                self.print_passwords(passwords);
                PasswordDeletionStatus::Multiple
            }
            else {
                match self.pw_core.delete_password(passwords[0].id.to_string().as_str()) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Error deleting password.");
                        exit(1);
                    }
                };
                PasswordDeletionStatus::Ok
            }
        }
    }
}
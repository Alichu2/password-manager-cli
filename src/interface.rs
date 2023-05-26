use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use password_manager::password_manager::{ PasswordManager, Password };
use std::process::exit;
use cli::cli::CLI;

pub struct PasswordManagerInterface {
    pw_core: PasswordManager,
    cli: CLI,
}

impl PasswordManagerInterface {
    pub fn new() -> Self {
        Self {
            pw_core: PasswordManager::new(),
            cli: CLI::new(),
        }
    }

    fn print_passwords(&self, passwords: Vec<Password>, key: &str, print_id: bool) {
        for (index, password) in passwords.iter().enumerate() {
            if print_id {
                println!("\n{} (ID = {}):", index, password.id);
            }
            else if passwords.len() > 1 {
                println!("\n{}:", index);
            }
            println!("  place = {}", &password.place);
            println!("  username = {}", &password.username);
            if password.encrypted {
                println!("  password = {}", match self.pw_core.decrypt(&password.password, key) {
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
        }
    }

    fn password_printing_manager(&self, passwords: Vec<Password>, print_id: bool, key: Option<String>) {
        let mut requires_key: bool = false;
        let number_passwords = passwords.len();

        for password in passwords.iter() {
            if password.encrypted {
                requires_key = true;
            }
        }

        let key: String = match key {
            Some(val) => val,
            None => {
                if requires_key {
                    self.get_key()
                }
                else {
                    String::new()
                }
            }
        };

        self.print_passwords(passwords, &key, print_id);
        println!("\n{} password(s) in total.", number_passwords);
    }

    fn verify_key(&self, key: &str) {
        if !self.pw_core.verify_key(&key) {
            println!("Incorrect key, try again.");
            exit(1);
        }
    }

    fn get_key(&self) -> String {
        let key = self.cli.get_password("Access Key: ");

        self.verify_key(&key);
        key
    }

    pub fn load_all_passwords(&self) {
        let result = self.pw_core.get_passwords("SELECT * FROM passwords;");

        match result {
            Ok(val) => {
                self.password_printing_manager(val, false, None);
            },
            Err(_) => {
                println!("An error occurred when loading your passwords. Please try again or report an issue.");
                exit(1);
            }
        }
    }

    pub fn load_password(&self, place: &str) {
        let result = self.pw_core.get_passwords(&("SELECT * FROM passwords WHERE place LIKE '%".to_string() + place + "%';"));

        match result {
            Ok(val) => {
                self.password_printing_manager(val, false, None)
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

    pub fn generate_password(&self, special_char: bool, upper_case: bool, digits: bool, length: usize) -> String {
        self.pw_core.generate(length, upper_case, digits, special_char)
    }

    pub fn generate_and_save(&self, special_char: bool, upper_case: bool, digits: bool, length: usize, uname: String, place: String, encrypt: bool) {
        let generated_password = self.generate_password(special_char, upper_case, digits, length);
        let saving_password: String;

        if encrypt {
            saving_password = self.pw_core.encrypt(&generated_password, &self.get_key());
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
        println!("\nSaved Password:\n  password = {}\n  username = {}\n  place = {}", generated_password, uname, place);
    }

    pub fn delete_password(&self, place: String) {
        let key = self.get_key(); // Make sure they have the authority to delete a password.

        let passwords = match self.pw_core.get_passwords(&("SELECT * FROM passwords WHERE place LIKE '%".to_string() + &place + "%';")) {
            Ok(val) => val,
            Err(_) => {
                println!("Error deleting password.");
                exit(1);
            }
        };

        if passwords.len() > 1 {
            self.password_printing_manager(passwords, true, Some(key));

            let id = self.cli.ask("Enter the ID of the password you want to delete:");

            match self.pw_core.delete_password(id.as_str()) {
                Ok(_) => (),
                Err(_) => {
                    println!("Error deleting password.");
                    exit(1);
                }
            }
        }
        else {
            match self.pw_core.delete_password(passwords[0].id.to_string().as_str()) {
                Ok(_) => (),
                Err(_) => {
                    println!("Error deleting password.");
                    exit(1);
                }
            };
        }
    }

    pub fn save_file_exists(&self) -> bool {
        self.pw_core.save_file_exists()
    }

    pub fn add_password(&self, password: &str, username: &str, place: &str, encrypt: bool) {
        let saving_password: String;

        if encrypt {
            saving_password = self.pw_core.encrypt(password, &self.get_key());
        }
        else {
            saving_password = password.to_string();
        }

        match self.pw_core.save_password(&saving_password, username, place, encrypt) {
            Ok(_) => (),
            Err(_) => {
                println!("Problem saving password. Try Again.");
                exit(1);
            }
        }

        println!("\nSaved Password:\n  password = {}\n  username = {}\n  place = {}", password, username, place);
    }

    pub fn create_backup(&self, path: PathBuf, encrypt: bool) {
        let key = self.get_key();
        let mut file_string: String = String::new();
        let passwords = match self.pw_core.get_passwords("SELECT * FROM passwords;") {
            Ok(val) => val,
            Err(_) => {
                println!("An error occurred trying to generate your backup file.");
                exit(1);
            }
        };

        for password in passwords.iter() {
            let save_password;

            if password.encrypted {
                save_password = match self.pw_core.decrypt(&password.password, &key) {
                    Ok(val) => val,
                    Err(_) => {
                        println!("Error decrypting.");
                        exit(1);
                    }
                };
            }
            else {
                save_password = password.password.clone();
            }
            file_string.push_str(&format!("{}|{}|{}\n", save_password, password.username, password.place));
        }

        let mut file = match File::create(path.join("password_backup.txt")) {
            Ok(val) => val,
            Err(_) => {
                println!("Error creating file.");
                exit(1);
            }
        };

        let file_contents;
        if encrypt {
            let file_key = self.cli.get_password("File key (used to encrypt and later decrypt file): ");
            if file_key == self.cli.get_password("Confirm file key: ") {
                file_contents = self.pw_core.encrypt(&file_string, &file_key);
            }
            else {
                println!("Different keys! Try again.");
                exit(1);
            }
        }
        else {
            file_contents = file_string;
        }

        match file.write_all((&file_contents).as_bytes()) {
            Ok(_) => (),
            Err(_) => {
                println!("Error writing to file");
                exit(1);
            }
        }
    }

    pub fn restore_backup(&self, file: PathBuf, encrypt: bool) {
        let key = self.get_key();
        let file_contents: String = match fs::read_to_string(file) {
            Ok(val) => val,
            Err(_) => {
                println!("Error reading backup file.");
                exit(1);
            }
        };
        let processing_string;

        if !file_contents.contains("|") {
            processing_string = match self.pw_core.decrypt(&file_contents, &self.cli.get_password("File key: ")) {
                Ok(val) => val.trim().to_string(),
                Err(_) => {
                    println!("Error decrypting file.");
                    exit(1);
                }
            }
        }
        else {
            processing_string = file_contents.trim().to_string();
        }


        for line in processing_string.split("\n") {
            let line_contents: Vec<&str> = line.split("|").collect();
            let save_password;

            if encrypt {
                save_password = self.pw_core.encrypt(line_contents[0], &key);
            }
            else {
                save_password = line_contents[0].to_string();
            }

            match self.pw_core.save_password(&save_password, line_contents[1], line_contents[2], encrypt) {
                Ok(_) => (),
                Err(_) => {
                    println!("Error saving password.");
                    exit(1);
                }
            }
        }
    }
}
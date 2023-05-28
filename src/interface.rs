use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use password_manager::password_manager::{ PasswordManager, Password };
use std::process::exit;
use cli::cli::CLI;

pub struct PasswordManagerInterface {
    pub pw_core: PasswordManager,
    cli: CLI,
}

impl PasswordManagerInterface {
    pub fn new() -> Self {
        Self {
            pw_core: PasswordManager::new(),
            cli: CLI::new(),
        }
    }

    fn print_passwords(&self, passwords: &Vec<Password>, key: String) {
        for (index, password) in passwords.iter().enumerate() {
            println!("\n{}:", index);
            println!("  place = {}", &password.place);
            println!("  username = {}", &password.username);
            if password.encrypted {
                println!("  password = {}", match self.pw_core.decrypt(&password.password, key.clone().as_str()) {
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

    fn password_printing_manager(&self, passwords: &Vec<Password>, key: Option<String>) {
        let mut requires_key: bool = false;
        let number_passwords = passwords.len();

        if number_passwords == 0 {
            println!("No passwords have been found.");
        }
        else {
            for password in passwords.iter() {
                if password.encrypted {
                    requires_key = true;
                }
            }

            let new_key: String;
            if requires_key {
                if &key == &None {
                    new_key = self.get_key();
                }
                else {
                    new_key = key.unwrap();
                }
            }
            else {
                new_key = String::new();
            }

            self.print_passwords(passwords, new_key);
            println!("\n{} password(s) in total.", number_passwords);
        }
    }

    fn get_key(&self) -> String {
        let mut key = String::new();

        while key.is_empty() {
            key = self.cli.prompt_password("Access Key: ");
        }
        key
    }

    pub fn load_all_passwords(&self) {
        let result = self.pw_core.get_all_passwords();

        self.password_printing_manager(&result, None);
    }

    pub fn load_password(&self, place: &str) {
        let result = self.pw_core.get_password(place);

        self.password_printing_manager(&result, None)
    }


    pub fn generate_and_save(&self, generated_password: String, uname: String, place: String, encrypt: bool) {
        if encrypt {
            self.pw_core.save_password(&generated_password, &uname, &place, encrypt, Some(&self.get_key()));
        }
        else {
            self.pw_core.save_password(&generated_password, &uname, &place, encrypt, None);
        }

        println!("\nSaved Password:\n  password = {}\n  username = {}\n  place = {}", generated_password, uname, place);
    }

    pub fn delete_password(&self, place: String) {
        let key = self.get_key(); // Make sure they have the authority to delete a password.
        let password = self.pw_core.get_password(&place);

        if password.len() == 0 {
            println!("No password found.")
        }
        else {
            self.password_printing_manager(&password, Some(key));
            if self.cli.prompt("Are you sure you want to delete this password? [y/n]:").unwrap().as_str() == "y" {
                self.pw_core.delete_password(password[0].place.as_str());
            }
            else {
                println!("Deletion aborted.");
            }
        }
    }

    pub fn add_password(&self, password: &str, username: &str, place: &str, encrypt: bool) {
        if encrypt {
            self.pw_core.save_password(password, username, place, encrypt, Some(&self.get_key()));
        }
        else {
            self.pw_core.save_password(password, username, place, encrypt, None);
        }

        println!("\nSaved Password:\n  password = {}\n  username = {}\n  place = {}", password, username, place);
    }

    pub fn create_backup(&self, path: PathBuf, encrypt: bool) {
        let key = self.get_key();
        let mut file_string: String = String::new();
        let passwords = self.pw_core.get_all_passwords();

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
            file_string.push_str(&format!("{},{},{}\n", save_password, password.username, password.place));
        }

        let mut file = match File::create(path.join(if encrypt { "password_backup.txt" } else { "password_backup.csv" })) {
            Ok(val) => val,
            Err(_) => {
                println!("Error creating file.");
                exit(1);
            }
        };

        let file_contents;
        if encrypt {
            let file_key = self.cli.prompt_password("File key (used to encrypt and later decrypt file): ");
            if file_key == self.cli.prompt_password("Confirm file key: ") {
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

        if !file_contents.contains(",") {
            processing_string = match self.pw_core.decrypt(&file_contents, &self.cli.prompt_password("File key: ")) {
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
            let line_contents: Vec<&str> = line.split(",").collect();

            if encrypt {
                self.pw_core.save_password(line_contents[0], line_contents[1], line_contents[2], encrypt, Some(&key));
            }
            else {
                self.pw_core.save_password(line_contents[0], line_contents[1], line_contents[2], encrypt, None);
            }
        }
    }
}
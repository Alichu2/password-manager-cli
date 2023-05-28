pub mod password_manager {
    use std::collections::HashMap;
    use std::fs;
    use rand::prelude::IteratorRandom;
    use std::path::PathBuf;
    use sqlite;
    use std::process::exit;
    use magic_crypt::{new_magic_crypt, MagicCryptTrait};
    use sqlite::Bindable;

    #[allow(unused_imports)]
    use dirs::home_dir;

    pub struct Password {
        pub password: String,
        pub username: String,
        pub place: String,
        pub encrypted: bool,
    }

    /*
    >> How does `access_check_plain` work? <<
    When a save file is created with --new-key , the key that the user enters is not saved. But to prevent the user accidentally saving
    a password with an incorrect key or to prevent malicious intent, a string (access_check_plain) is encrypted with the key when the
    save file is created. Whenever the user wants to perform an action, the saved ciphertext gets decrypted with the user entered key
    and compared with access_check_plain to make sure it is the correct key.
     */

    // TODO: Check that having plaintext and ciphertext makes it easy to figure out the key.

    static CONF_ACCESS_CHECK: &str = "access_check_cipher";

    pub struct PasswordManager {
        access_check_plain: &'static str,
    }

    impl PasswordManager {
        pub fn new() -> Self {
            Self {
                access_check_plain: "test string to encrypt",
            }
        }

        #[cfg(not(debug_assertions))]
        fn get_home_path(&self) -> PathBuf {
            match home_dir() {
                None => {
                    println!("Error: Home Directory Path Not Found.");
                    exit(1);
                }
                Some(path) => path
            }
        }

        #[cfg(debug_assertions)]
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
                Ok(val) => val,
                Err(_) => {
                    println!("Failed to get connection. Is the save file ok?");
                    exit(1);
                }
            }
        }

        fn execute_sql<T: Bindable>(&self, query: &str, params: T, empty_params: bool) {
            let connection = self.get_sqlite_connection();
            let mut statement = connection.prepare(query).expect("Error preparing connection.");

            if !empty_params {
                statement.bind(params).expect("Error running sql.");
            }

            statement.next().expect("Error running sql.");
        }

        pub fn read_from_sql<T: Bindable>(&self, query: &str, params: T, columns: Vec<&str>, empty_params: bool) -> Vec<HashMap<String, String>> {
            let connection = self.get_sqlite_connection();
            let mut statement = connection.prepare(query).expect("Error preparing sql connection.");
            let mut result = Vec::new();

            if !empty_params {
                statement.bind(params).expect("Error binding sql.");
            }

            while let Ok(sqlite::State::Row) = statement.next() {
                let mut row_result: HashMap<String, String> = HashMap::new();

                for column in columns.iter() {
                    let value = statement.read::<String, _>(column.clone()).expect("Error reading value.");
                    row_result.insert(String::from(column.clone()), value);
                }
                result.push(row_result);
            }
            result
        }

        pub fn save_new_key(&self, key: String) {
            let access_check_cipher = self.encrypt(self.access_check_plain, &key);

            self.execute_sql("INSERT INTO config (name, value) VALUES (:key, :cipher);",
                             &[(":key", CONF_ACCESS_CHECK), (":cipher", &access_check_cipher)][..], false);
        }

        pub fn save_password(&self, password: &str, username: &str, place: &str, encrypt: bool, key: Option<&str>) {
            let save_password;

            if encrypt {
                save_password = self.encrypt(password, key.unwrap());
            }
            else {
                save_password = password.to_string();
            }

            self.execute_sql("INSERT OR REPLACE INTO passwords (password, username, place, is_encrypted) VALUES (:password, :username, :place, :encrypted);",
            &[(":password", save_password.as_str()), (":username", username), (":place", place), (":encrypted", &(encrypt as usize).to_string())][..], false)
        }

        pub fn delete_password(&self, place: &str) {
            self.execute_sql(&("DELETE FROM passwords WHERE place = :place;"), (":place", place), false)
        }

        pub fn unpack_passwords(&self, packed_passwords: Vec<HashMap<String, String>>) -> Vec<Password> {
            let mut unpacked_passwords = Vec::new();

            for packed_password in packed_passwords.iter() {
                unpacked_passwords.push(Password {
                    password: packed_password.get("password").expect("Missing value for `password`.").clone(),
                    place: packed_password.get("place").expect("Missing value for `place`.").clone(),
                    username: packed_password.get("username").expect("Missing value for `username`.").clone(),
                        encrypted: &(packed_password.get("is_encrypted").expect("Missing value for `is_encrypted`.").clone()) == "1",
                })
            }

            unpacked_passwords
        }

        pub fn create_new_save_file(&self, new_key: &str) {
            if !self.save_file_exists() {
            match self.create_save_file() {
                Ok(_) => (),
                Err(_) => {
                    println!("Error creating save file.");
                    exit(1);
                }
            }
            self.save_new_key(new_key.to_string());
        }
        else {
            println!("Save file and key already exists. Cannot regenerate.");
        }
        }

        pub fn generate_password(&self, length: usize, include_uppercase: bool, include_digits: bool, include_special: bool) -> String {
            let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
            let digit_chars = "0123456789";
            let special_chars = "!@#$%^&*()-_=+[]{}<>/?";

            let mut char_set = String::new();

            char_set.push_str(lowercase_chars);

            if include_uppercase { char_set.push_str(lowercase_chars.to_uppercase().as_str()); }
            if include_digits { char_set.push_str(digit_chars); }
            if include_special { char_set.push_str(special_chars); }

            let mut result = String::new();

            for _ in 0..length {
                let next_char: char = char_set.chars().choose(&mut rand::thread_rng())
                    .expect("Could not generate password (Error Rand Select). If this issue persists, please create a github issue at https://github.com/Alichu2/password-manager-cli");

                result.push_str(next_char.to_string().as_str());
            }

            result
        }

        pub fn get_all_passwords(&self) -> Vec<Password> {
            let result = self.read_from_sql("SELECT * FROM passwords;", ("", ""),
                                            vec!["password", "place", "username", "is_encrypted"], true);

            self.unpack_passwords(result)
        }

        pub fn get_password(&self, place: &str) -> Vec<Password> {
            let packed_passwords = self.read_from_sql("SELECT * FROM passwords WHERE place = :place;", (":place", place),
                                                      vec!["password", "username", "place", "is_encrypted"], false);

            self.unpack_passwords(packed_passwords)
        }

        pub fn create_save_file(&self) -> Result<(), ()> {
            match fs::create_dir_all(self.get_save_dir_path().display().to_string()) {
                Ok(_) => {
                    self.execute_sql("CREATE TABLE passwords (password TEXT, username TEXT, place TEXT PRIMARY KEY, is_encrypted NUMBER);", ("", ""), true);
                    self.execute_sql("CREATE TABLE config (name TEXT, value TEXT);", ("", ""), true);
                    Ok(())
                },
                Err(_) => Err(())
            }
        }

        pub fn verify_key(&self, key: &str) -> bool {
            let result = self.read_from_sql("SELECT * FROM config WHERE name = :key", (":key", CONF_ACCESS_CHECK), vec!["value"], false);
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

            decrypted_string == self.access_check_plain
        }

        pub fn encrypt(&self, plaintext: &str, key: &str) -> String {
            let mc = new_magic_crypt!(key, 256);

            mc.encrypt_str_to_base64(plaintext)
        }

        pub fn decrypt(&self, ciphertext: &str, key: &str) -> Result<String, ()> {
            let mc = new_magic_crypt!(key, 256);

            match mc.decrypt_base64_to_string(ciphertext) {
                Ok(result) => Ok(result),
                Err(_) => Err(())
            }
        }
    }
}
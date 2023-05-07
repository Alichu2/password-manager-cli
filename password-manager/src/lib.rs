mod password_manager {
    use std::collections::HashMap;
    use rand::prelude::IteratorRandom;
    use std::path::PathBuf;
    use dirs::home_dir;
    use sqlite;
    use std::process::exit;

    pub struct PasswordManager {}

    impl PasswordManager {
        pub fn new() -> Self {
            Self {}
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

        fn get_save_file_path(&self) -> String {
            self.get_save_dir_path().join("data.sqlite").display().to_string()
        }

        fn get_sqlite_connection(&self) -> sqlite::Connection {
            match sqlite::open(&self.get_save_file_path()) {
                Ok(connection) => connection,
                Err(_) => {
                    println!("An error occurred when trying to open the save file.");
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

        fn read_sql_data(&self, fields: Vec<&str>, search_query: &str) -> Result<Vec<HashMap<&str, String>>, ()> {
            let connection = self.get_sqlite_connection();

            match connection.prepare(search_query) {
                Ok(statement) => {
                    let mut result_data: Vec<HashMap<&str, String>> = Vec::new();

                    while let Ok(sqlite::State::Row) = statement.next() {
                        let mut row_data: HashMap<&str, String> = HashMap::new();

                        for field in fields.iter() {
                            let value = statement.read::<String, _>(field).expect("Error reading values from save file.");
                            row_data.insert(field, value);
                        }

                        result_data.push(row_data);
                    }

                    Ok(result_data)
                },
                Err(_) => Err(())
            }
        }

        pub fn generate(length: usize, include_uppercase: bool, include_digits: bool, include_special: bool) -> String {
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
    }
}

pub mod password_interface {

}
use anyhow::{bail, Result};
use rand::prelude::IteratorRandom;
use sqlx::Executor;
use std::fmt;
use std::process::exit;

use crate::consts::{LOWERCASE_CHARACTERS, NUMBERS, SPECIAL_CHARACTERS};
use crate::database::get_sqlite_connection;
use crate::security::{decrypt, encrypt};

#[derive(sqlx::FromRow, Clone)]
pub struct Password {
    pub password: String,
    pub username: String,
    pub place: String,
    pub encrypted: i32,
}

pub struct PasswordBuilder {
    pub username: String,
    pub place: String,
    options: PasswordBuildOptions,
}

#[derive(Clone, Copy)]
pub struct PasswordBuildOptions {
    pub length: usize,
    pub use_special: bool,
    pub use_numbers: bool,
    pub use_upper: bool,
}

impl PasswordBuilder {
    pub fn from(username: String, place: String, options: PasswordBuildOptions) -> Self {
        Self {
            username,
            place,
            options,
        }
    }

    pub fn to_password(&self) -> Password {
        let password = Self::generate_valid_password(self.options);

        Password::new(self.username.clone(), self.place.clone(), password)
    }

    pub fn generate_valid_password(options: PasswordBuildOptions) -> String {
        let mut password = String::new();
        let mut correct = false;

        while !correct {
            password = Self::generate_password(options);
            correct = Self::verify_password(options, &password);
        }

        password
    }

    pub fn verify_password(options: PasswordBuildOptions, password: &str) -> bool {
        let mut is_not_correct = false;
        is_not_correct = is_not_correct || !(password.len() == options.length);
        if options.use_special {
            is_not_correct = is_not_correct || !Self::contains_char(SPECIAL_CHARACTERS, password);
        }
        if options.use_numbers {
            is_not_correct = is_not_correct || !Self::contains_char(NUMBERS, password);
        }
        if options.use_upper {
            is_not_correct = is_not_correct
                || !Self::contains_char(&LOWERCASE_CHARACTERS.to_uppercase(), password);
        }

        !is_not_correct
    }

    fn contains_char(charset: &str, text: &str) -> bool {
        for char in charset.chars() {
            if text.contains(char) {
                return true;
            }
        }

        false
    }

    pub fn generate_password(options: PasswordBuildOptions) -> String {
        let mut char_set = LOWERCASE_CHARACTERS.to_string();

        if options.use_upper {
            char_set.push_str(LOWERCASE_CHARACTERS.to_uppercase().as_str());
        }
        if options.use_numbers {
            char_set.push_str(NUMBERS);
        }
        if options.use_special {
            char_set.push_str(SPECIAL_CHARACTERS);
        }

        let mut result = String::new();

        for _ in 0..options.length {
            let next_char: char = char_set
                .chars()
                .choose(&mut rand::thread_rng())
                .expect("Could not generate password (Error Rand Select).");

            result.push_str(next_char.to_string().as_str());
        }

        result
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\tpalce = {}\n\tusername = {}\n\tpassword = {}",
            self.place,
            self.username,
            self.password.clone()
        )
    }
}

impl Password {
    pub fn new(username: String, place: String, password: String) -> Self {
        Self {
            password,
            place,
            encrypted: 0,
            username,
        }
    }

    pub async fn from(place: String) -> Self {
        let db_conn = get_sqlite_connection();

        let possibilities = sqlx::query_as::<_, Self>("SELECT * FROM passwords WHERE place = ?;")
            .bind(&place)
            .fetch_all(&mut db_conn.await)
            .await
            .expect("Error reading password from database.");

        if possibilities.len() == 0 {
            println!("No passwords found.");
            exit(0);
        } else {
            possibilities[0].clone()
        }
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypted == 1
    }

    pub fn decrypt_password(&mut self, key: &str) -> Result<()> {
        if self.is_encrypted() {
            self.password = decrypt(&self.password, key)?;
            self.encrypted = 0;
        } else {
            bail!("Attempting to decrypt an already decrypted password. Ignoring.")
        }

        Ok(())
    }

    pub fn encrypt_password(&mut self, key: &str) -> Result<()> {
        if !self.is_encrypted() {
            self.password = encrypt(&self.password, key);
            self.encrypted = 1;
        } else {
            bail!("Attempting to decrypt an already decrypted password. Ignoring.")
        }

        Ok(())
    }

    pub fn to_csv_row(&mut self) -> String {
        format!(
            "{},{},{}\n",
            self.place,
            self.username,
            self.password.clone()
        )
    }

    pub async fn delete(&self) {
        let mut db_conn = get_sqlite_connection().await;

        db_conn
            .execute(sqlx::query("DELETE FROM passwords WHERE place = ?;").bind(&self.place))
            .await
            .expect("Error deleting password.");
    }

    pub async fn save(&self) -> Result<()> {
        let mut db_conn = get_sqlite_connection().await;

        db_conn.execute(
            sqlx::query("INSERT INTO passwords (place, password, username, encrypted) VALUES (?, ?, ?, ?);")
                .bind(&self.place)
                .bind(&self.password)
                .bind(&self.username)
                .bind(self.encrypted))
        .await?;

        Ok(())
    }
}

pub async fn get_all_passwords() -> Vec<Password> {
    let mut db_conn = get_sqlite_connection().await;

    let passwords = sqlx::query_as::<_, Password>("SELECT * FROM passwords;")
        .fetch_all(&mut db_conn)
        .await
        .expect("Error reading passwords from database.");

    passwords
}

pub async fn get_all_decrypted_passwords(key: &str) -> Vec<Password> {
    let mut all_passwords = get_all_passwords().await;

    for password in all_passwords.iter_mut() {
        if password.is_encrypted() {
            password
                .decrypt_password(key)
                .expect("Error decrypting one of the passwords.");
        }
    }

    all_passwords
}

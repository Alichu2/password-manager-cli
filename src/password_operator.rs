use anyhow::{bail, Result};
use rand::prelude::IteratorRandom;
use sqlx::Executor;
use std::fmt;

use crate::consts::{LOWERCASE_CHARACTERS, NUMBERS, SPECIAL_CHARACTERS};
use crate::database::get_sqlite_connection;
use crate::security::{decrypt, encrypt};

#[derive(sqlx::FromRow)]
pub struct Password {
    pub password: Option<String>,
    pub username: String,
    pub place: String,
    pub encrypted: i32,
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.password.is_some() {
            write!(
                f,
                "\tpalce = {}\n\tusername = {}\n\tpassword = {}",
                self.place,
                self.username,
                self.password.clone().unwrap()
            )
        } else {
            write!(
                f,
                "\tpalce = {}\n\tusername = {}\n\tpassword = not generated",
                self.place, self.username
            )
        }
    }
}

impl Password {
    pub fn new(username: String, place: String, password: Option<String>) -> Self {
        Self {
            password,
            place,
            encrypted: 0,
            username,
        }
    }

    pub async fn from(place: String) -> Self {
        let db_conn = get_sqlite_connection();

        sqlx::query_as::<_, Self>("SELECT * FROM passwords WHERE place = ?;")
            .bind(&place)
            .fetch_one(&mut db_conn.await)
            .await
            .expect("Error reading password from database.")
    }

    pub async fn generate_and_attach_password(
        &mut self,
        length: usize,
        use_special: bool,
        use_numbers: bool,
        use_upper: bool,
    ) {
        self.password = Some(Self::generate_valid_password(
            length,
            use_special,
            use_numbers,
            use_upper,
        ));
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypted == 1
    }

    pub fn generate_valid_password(
        length: usize,
        use_special: bool,
        use_numbers: bool,
        use_upper: bool,
    ) -> String {
        let mut password = Self::generate_password(length, use_special, use_numbers, use_upper);
        let mut correct =
            Self::verify_password(length, use_special, use_numbers, use_upper, &password);

        while !correct {
            password = Self::generate_password(length, use_special, use_numbers, use_upper);
            correct = Self::verify_password(length, use_special, use_numbers, use_upper, &password);
        }

        password
    }

    pub fn verify_password(
        length: usize,
        use_special: bool,
        use_numbers: bool,
        use_upper: bool,
        password: &str,
    ) -> bool {
        let mut is_not_correct = false;
        is_not_correct = is_not_correct || !(password.len() == length);
        if use_special {
            is_not_correct = is_not_correct || !Self::contains_char(SPECIAL_CHARACTERS, password);
        }
        if use_numbers {
            is_not_correct = is_not_correct || !Self::contains_char(NUMBERS, password);
        }
        if use_upper {
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

    pub fn generate_password(
        length: usize,
        use_special: bool,
        use_numbers: bool,
        use_upper: bool,
    ) -> String {
        let lowercase_chars = "abcdefghijklmnopqrstuvwxyz";
        let digit_chars = "0123456789";
        let special_chars = "!@#$%^&*()-_=+[]{}<>/?";

        let mut char_set = String::new();

        char_set.push_str(lowercase_chars);

        if use_upper {
            char_set.push_str(lowercase_chars.to_uppercase().as_str());
        }
        if use_numbers {
            char_set.push_str(digit_chars);
        }
        if use_special {
            char_set.push_str(special_chars);
        }

        let mut result = String::new();

        for _ in 0..length {
            let next_char: char = char_set
                .chars()
                .choose(&mut rand::thread_rng())
                .expect("Could not generate password (Error Rand Select).");

            result.push_str(next_char.to_string().as_str());
        }

        result
    }

    pub fn decrypt_password(&mut self, key: &str) -> Result<()> {
        if self.is_encrypted() {
            if self.password.is_some() {
                self.password = Some(decrypt(&self.password.clone().unwrap(), key)?);
                self.encrypted = 0;
            } else {
                bail!("Cannot decrypt a non existing password.");
            }
        } else {
            bail!("Attempting to decrypt an already decrypted password. Ignoring.")
        }

        Ok(())
    }

    pub fn encrypt_password(&mut self, key: &str) -> Result<()> {
        if !self.is_encrypted() {
            if self.password.is_some() {
                self.password = Some(encrypt(&self.password.clone().unwrap(), key));
                self.encrypted = 1;
            } else {
                bail!("Cannot decrypt a non existing password.");
            }
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
            self.password.clone().unwrap_or(String::from("no password"))
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
        if self.password.is_some() {
            let mut db_conn = get_sqlite_connection().await;

            db_conn.execute(
                sqlx::query("INSERT INTO passwords (place, password, username, encrypted) VALUES (?, ?, ?, ?);")
                    .bind(&self.place)
                    .bind(&self.password.clone().unwrap())
                    .bind(&self.username)
                    .bind(self.encrypted))
            .await?;
        } else {
            bail!("Could not save password because no password exists.");
        }

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

use rand::seq::SliceRandom;
use std::fmt;

use crate::consts::{LOWERCASE_CHARACTERS, NUMBERS, SPECIAL_CHARACTERS};
use crate::database::queries::DatabaseInterface;
use crate::errors::Error;
use crate::utils::{decrypt, encrypt};

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

#[derive(Clone)]
pub struct PasswordBuildOptions {
    pub length: usize,
    pub use_special: bool,
    pub use_numbers: bool,
    pub use_upper: bool,
    pub exclude_char: Vec<char>,
}

impl PasswordBuilder {
    pub fn from(username: String, place: String, options: PasswordBuildOptions) -> Self {
        Self {
            username,
            place,
            options,
        }
    }

    fn build_charset(options: &PasswordBuildOptions) -> Vec<char> {
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

        char_set
            .chars()
            .filter(|char| !options.exclude_char.contains(char))
            .collect::<Vec<_>>()
    }

    pub fn generate_password(options: PasswordBuildOptions) -> String {
        let char_set = Self::build_charset(&options);
        let mut result = String::new();

        for _ in 0..options.length {
            let next_char = char_set.choose(&mut rand::thread_rng()).unwrap();

            result.push_str(next_char.to_string().as_str());
        }

        result
    }
}

impl Into<Password> for PasswordBuilder {
    fn into(self) -> Password {
        let password = PasswordBuilder::generate_password(self.options);

        Password::new(self.username, self.place, password)
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_encrypted() {
            write!(
                f,
                "\tpalce = {}\n\tusername = {}",
                self.place, self.username
            )
        } else {
            write!(
                f,
                "\tpalce = {}\n\tusername = {}\n\tpassword = {}",
                self.place, self.username, self.password
            )
        }
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

    pub async fn from(place: String, conn: &mut DatabaseInterface) -> Result<Self, Error> {
        let password = conn.get_password(&place).await?;

        if password.len() == 0 {
            Err(Error::NoPassword(place))
        } else {
            Ok(password.into_iter().nth(0).unwrap())
        }
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypted == 1
    }

    pub fn decrypt_password(&mut self, key: &str) -> Result<(), Error> {
        if self.is_encrypted() {
            self.password = decrypt(&self.password, key)?;
            self.encrypted = 0;
        }

        Ok(())
    }

    pub fn encrypt_password(&mut self, key: &str) {
        if !self.is_encrypted() {
            self.password = encrypt(&self.password, key);
            self.encrypted = 1;
        }
    }

    pub fn to_csv_row(&self) -> String {
        format!("{},{},{}\n", self.place, self.username, self.password)
    }
}

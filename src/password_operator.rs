use rand::prelude::IteratorRandom;
use std::fmt;

#[allow(unused_imports)]
use dirs::home_dir;

use crate::consts::{LOWERCASE_CHARACTERS, NUMBERS, SPECIAL_CHARACTERS};

pub struct Password {
    pub password: Option<String>,
    pub username: String,
    pub place: String,
    pub encrypted: bool,
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
    pub fn new(username: String, place: String, encrypted: bool, password: Option<String>) -> Self {
        Self {
            password,
            place,
            encrypted,
            username,
        }
    }

    pub async fn from(_place: String) -> Self {
        unimplemented!();
    }

    pub async fn generate_and_attach_password(
        &mut self,
        length: usize,
        use_special: bool,
        use_numbers: bool,
        use_upper: bool,
    ) {
        let mut password = Self::generate_password(length, use_special, use_numbers, use_upper);
        let mut correct =
            Self::verify_password(length, use_special, use_numbers, use_upper, &password);

        while !correct {
            password = Self::generate_password(length, use_special, use_numbers, use_upper);
            correct = Self::verify_password(length, use_special, use_numbers, use_upper, &password);
        }

        self.password = Some(password);
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

    pub fn delete(&self) {
        unimplemented!();
    }
}

pub fn get_all_passwords() -> Vec<Password> {
    todo!();
}

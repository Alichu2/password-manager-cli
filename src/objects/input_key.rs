use crate::{
    consts::{FAIL_KEY_ASK, FIRST_KEY_ASK, HASH_COST},
    database::{manager::get_sqlite_connection, queries},
};
use anyhow::Result;
use bcrypt::hash;
use bcrypt::verify;
use rpassword::prompt_password;

use super::query_results::{ConfigItem, ConfigParams};

pub struct InputKey {
    key: Option<String>,
}

impl InputKey {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn new_pre_ask() -> Result<Self> {
        let key = Self::ask_key(false)?;

        Ok(Self { key: Some(key) })
    }

    pub async fn get_key(&mut self) -> Result<String> {
        if self.key.is_none() {
            let new = Self::ask_valid_key().await?;

            self.key = Some(new.clone());
            Ok(new)
        } else {
            Ok(self.key.clone().unwrap())
        }
    }

    async fn ask_valid_key() -> Result<String> {
        let mut key = String::new();
        let mut is_valid = false;
        let mut has_tried = false;

        while !is_valid {
            key = Self::ask_key(has_tried)?;
            is_valid = Self::verify_key(&key).await?;
            has_tried = true;
        }

        Ok(key)
    }

    fn ask_key(first: bool) -> Result<String> {
        let propt = if first { FIRST_KEY_ASK } else { FAIL_KEY_ASK };
        let answer = prompt_password(propt)?;

        Ok(answer)
    }

    async fn verify_key(key: &str) -> Result<bool> {
        let encrypted = queries::get_setting(ConfigParams::AccessCheck).await?;
        let is_valid = verify(key, &encrypted.value)?;

        Ok(is_valid)
    }

    pub async fn save(&mut self, force: bool) -> Result<()> {
        let hash = Self::hashed_asked_key()?;
        let setting = ConfigItem {
            name: ConfigParams::AccessCheck,
            value: hash,
        };
        let mut conn = get_sqlite_connection().await;

        if force {
            queries::force_set_setting(setting, &mut conn).await?;
        } else {
            queries::set_setting(setting, &mut conn).await?;
        }

        Ok(())
    }

    fn hashed_asked_key() -> Result<String> {
        let key = Self::ask_key(false)?;
        let hash = hash(key, HASH_COST)?;

        Ok(hash)
    }

    pub async fn saved_key_exists() -> Result<bool> {
        todo!()
    }
}

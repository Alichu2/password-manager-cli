use std::io::{BufRead, Write};

use crate::{
    consts::{FAIL_KEY_ASK, FIRST_KEY_ASK, HASH_COST},
    database::{manager::get_sqlite_connection, queries},
};
use anyhow::Result;
use bcrypt::hash;
use bcrypt::verify;
use rpassword::prompt_password_from_bufread;

use super::query_results::{ConfigItem, ConfigParams};

pub async fn ask_valid_key<R: BufRead, W: Write>(read: &mut R, write: &mut W) -> Result<String> {
    let mut key = String::new();
    let mut is_valid = false;
    let mut has_tried = false;

    while !is_valid {
        key = ask_key(read, write, has_tried)?;
        is_valid = verify_key(&key).await?;
        has_tried = true;
    }

    Ok(key)
}

fn ask_key<R: BufRead, W: Write>(read: &mut R, write: &mut W, first: bool) -> Result<String> {
    let propt = if first { FIRST_KEY_ASK } else { FAIL_KEY_ASK };
    let answer = prompt_password_from_bufread(read, write, propt)?;

    Ok(answer)
}

async fn verify_key(key: &str) -> Result<bool> {
    let encrypted = queries::get_setting(ConfigParams::AccessCheck).await?;
    let is_valid = verify(key, &encrypted.value)?;

    Ok(is_valid)
}

pub async fn save<R: BufRead, W: Write>(read: &mut R, write: &mut W, force: bool) -> Result<()> {
    let hash = hashed_asked_key(read, write)?;
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

fn hashed_asked_key<R: BufRead, W: Write>(read: &mut R, write: &mut W) -> Result<String> {
    let key = ask_key(read, write, false)?;
    let hash = hash(key, HASH_COST)?;

    Ok(hash)
}

pub async fn saved_key_exists() -> Result<bool> {
    todo!()
}

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn get_key() {}
// }

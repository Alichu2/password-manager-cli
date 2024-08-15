use crate::consts::{CONF_ACCESS_CHECK, HASH_COST};
use crate::database::get_sqlite_connection;
use anyhow::Result;
use bcrypt::{hash, verify};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use sqlx::Executor;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ConfigItem {
    pub name: String,
    pub value: String,
}

pub async fn verify_key(key: &str) -> bool {
    let result = sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
        .bind(CONF_ACCESS_CHECK)
        .fetch_one(&mut get_sqlite_connection().await)
        .await
        .expect("Error reading key from database.");

    verify(key, &result.value).expect("Error verifying key.")
}

pub fn encrypt(plaintext: &str, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);

    mc.encrypt_str_to_base64(plaintext)
}

pub fn decrypt(ciphertext: &str, key: &str) -> Result<String> {
    let mc = new_magic_crypt!(key, 256);

    Ok(mc.decrypt_base64_to_string(ciphertext)?)
}

pub async fn save_new_key(key: String) {
    let hashed_key = hash(&key, HASH_COST).unwrap();
    let query = sqlx::query("INSERT INTO config (name, value) VALUES (?, ?);")
        .bind(CONF_ACCESS_CHECK)
        .bind(&hashed_key);
    let mut database_connection = get_sqlite_connection().await;

    database_connection
        .execute(query)
        .await
        .expect("Error saving new key.");
}

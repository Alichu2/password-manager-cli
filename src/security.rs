use crate::database::manager::get_sqlite_connection;
use crate::objects::query_results::ConfigParams;
use crate::{consts::HASH_COST, errors::Error};
use bcrypt::hash;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use sqlx::Executor;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ConfigItem {
    pub name: String,
    pub value: String,
}

pub fn encrypt(plaintext: &str, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);

    mc.encrypt_str_to_base64(plaintext)
}

pub fn decrypt(ciphertext: &str, key: &str) -> Result<String, Error> {
    let mc = new_magic_crypt!(key, 256);

    mc.decrypt_base64_to_string(ciphertext)
        .map_err(|err| Error::BadDecryption(err))
}

pub async fn save_new_key(key: String) {
    let hashed_key = hash(&key, HASH_COST).unwrap();
    let query = sqlx::query("INSERT INTO config (name, value) VALUES (?, ?);")
        .bind(ConfigParams::AccessCheck)
        .bind(&hashed_key);
    let mut database_connection = get_sqlite_connection().await;

    database_connection
        .execute(query)
        .await
        .expect("Error saving new key.");
}

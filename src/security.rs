use crate::database::objects::{ConfigItem, ConfigParams};
use crate::database::queries::DatabaseInterface;
use crate::{consts::HASH_COST, errors::Error};
use bcrypt::hash;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

pub fn encrypt(plaintext: &str, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);

    mc.encrypt_str_to_base64(plaintext)
}

pub fn decrypt(ciphertext: &str, key: &str) -> Result<String, Error> {
    let mc = new_magic_crypt!(key, 256);

    mc.decrypt_base64_to_string(ciphertext)
        .map_err(|err| Error::BadDecryption(err))
}

pub async fn save_new_key(key: &str, conn: &mut DatabaseInterface) -> Result<(), Error> {
    let hashed_key = hash(key, HASH_COST).map_err(|err| Error::HashError(err))?;
    let setting = ConfigItem {
        name: ConfigParams::AccessCheck,
        value: hashed_key,
    };

    conn.set_setting(setting).await?;

    Ok(())
}

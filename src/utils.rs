use crate::consts::communications::{CONFIRM_KEY, ENTER_KEY, ERROR_CONFIRMING_KEY, WRONG_KEY};
use crate::consts::{BACKUP_FILE_NAME, HASH_COST};
use crate::database::objects::{ConfigItem, ConfigParams};
use crate::database::queries::DatabaseInterface;
use crate::errors::Error;
use crate::password::Password;
use bcrypt::{hash, verify};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use rpassword::prompt_password;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub async fn save_key(conn: &mut DatabaseInterface) -> Result<(), Error> {
    let key: String;

    loop {
        let entered_key = prompt_password(ENTER_KEY).map_err(|_| Error::ReadError)?;
        let confirmation_key = prompt_password(CONFIRM_KEY).map_err(|_| Error::ReadError)?;

        if entered_key == confirmation_key {
            key = entered_key;
            break;
        } else {
            println!("{}", ERROR_CONFIRMING_KEY);
        }
    }

    let config = ConfigItem {
        name: ConfigParams::AccessCheck,
        value: key,
    };

    conn.set_setting(config).await?;

    Ok(())
}

pub async fn ask_valid_key(conn: &mut DatabaseInterface) -> Result<String, Error> {
    let setting = conn.get_setting(ConfigParams::AccessCheck).await?;

    loop {
        let key = prompt_password(ENTER_KEY).map_err(|_| Error::ReadError)?;
        let verification = verify(&key, &setting.value).map_err(|_| Error::VerificationError)?;

        if verification {
            return Ok(key);
        } else {
            println!("{}", WRONG_KEY);
        }
    }
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

pub async fn save_new_key(key: &str, conn: &mut DatabaseInterface) -> Result<(), Error> {
    let hashed_key = hash(key, HASH_COST).map_err(|err| Error::HashError(err))?;
    let setting = ConfigItem {
        name: ConfigParams::AccessCheck,
        value: hashed_key,
    };

    conn.set_setting(setting).await?;

    Ok(())
}

pub fn create_backup(location: &PathBuf, passwords: &Vec<Password>) -> Result<(), Error> {
    let mut result_string = String::from("place,username,password\n");

    for password in passwords {
        result_string.push_str(&password.to_csv_row());
    }

    let mut file =
        fs::File::create(location.join(BACKUP_FILE_NAME)).expect("Error creating backup file.");
    file.write_all(result_string.as_bytes())
        .expect("Error writing backup file.");

    Ok(())
}

use bcrypt::verify;
use rpassword::prompt_password;

use crate::consts::communications::{CONFIRM_KEY, ENTER_KEY, ERROR_CONFIRMING_KEY, WRONG_KEY};
use crate::database::objects::{ConfigItem, ConfigParams};
use crate::database::queries::DatabaseInterface;
use crate::errors::Error;

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

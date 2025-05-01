use anyhow::Result;
use bcrypt::verify;
use rpassword::prompt_password_from_bufread;
use std::io::{BufRead, Write};

use crate::consts::communications::{CONFIRM_KEY, ENTER_KEY, ERROR_CONFIRMING_KEY, WRONG_KEY};
use crate::database::queries::DatabaseInterface;
use crate::objects::query_results::{ConfigItem, ConfigParams};

pub async fn save_key<R: BufRead, W: Write>(
    conn: &mut DatabaseInterface,
    read: &mut R,
    write: &mut W,
) -> Result<()> {
    let key: String;

    loop {
        let entered_key = prompt_password_from_bufread(read, write, ENTER_KEY)?;
        let confirmation_key = prompt_password_from_bufread(read, write, CONFIRM_KEY)?;

        if entered_key == confirmation_key {
            key = entered_key;
            break;
        } else {
            writeln!(write, "{}", ERROR_CONFIRMING_KEY);
        }
    }

    let config = ConfigItem {
        name: ConfigParams::AccessCheck,
        value: key,
    };

    conn.set_setting(config).await?;

    Ok(())
}

pub async fn ask_valid_key<R: BufRead, W: Write>(
    conn: &mut DatabaseInterface,
    read: &mut R,
    write: &mut W,
) -> Result<String> {
    let setting = conn.get_setting(ConfigParams::AccessCheck).await?;

    loop {
        let key = prompt_password_from_bufread(read, write, ENTER_KEY)?;

        if verify(&key, &setting.value)? {
            return Ok(key);
        } else {
            write!(write, "{}", WRONG_KEY);
        }
    }
}

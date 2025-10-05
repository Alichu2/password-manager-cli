use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::consts::BACKUP_FILE_NAME;
use crate::errors::Error;
use crate::password_operator::Password;

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

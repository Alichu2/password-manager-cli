use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::consts::BACKUP_FILE_NAME;
use crate::database::queries::DatabaseInterface;
use crate::errors::Error;
use crate::password_operator::get_all_decrypted_passwords;

pub async fn create_backup(
    location: &PathBuf,
    key: &str,
    conn: &mut DatabaseInterface,
) -> Result<(), Error> {
    let all_passwords = get_all_decrypted_passwords(key, conn).await?;
    let mut result_string = String::from("place,username,password\n");

    for password in all_passwords.iter() {
        result_string.push_str(&password.to_csv_row());
    }

    let mut file =
        fs::File::create(location.join(BACKUP_FILE_NAME)).expect("Error creating backup file.");
    file.write_all(result_string.as_bytes())
        .expect("Error writing backup file.");

    Ok(())
}

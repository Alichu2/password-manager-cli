use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::consts::BACKUP_FILE_NAME;
use crate::password_operator::get_all_decrypted_passwords;

pub async fn create_backup(location: &mut PathBuf, key: &str) {
    let mut all_passwords = get_all_decrypted_passwords(key).await;
    let mut result_string = String::from("place,username,password\n");

    for password in all_passwords.iter_mut() {
        result_string.push_str(&password.to_csv_row());
    }

    location.push(BACKUP_FILE_NAME);

    let mut file = fs::File::create(location).expect("Error creating backup file.");
    file.write_all(result_string.as_bytes())
        .expect("Error writing backup file.");
}

pub fn restore_backup(file: String) {
    unimplemented!()
}

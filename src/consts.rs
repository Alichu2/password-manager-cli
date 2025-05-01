pub const CONF_ACCESS_CHECK: &str = "access_key_hash";
pub const HASH_COST: u32 = 8;
pub const LOWERCASE_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub const NUMBERS: &str = "0123456789";
pub const SPECIAL_CHARACTERS: &str = "!@#$%^&*()-_=+[]{}<>/?";
pub const BACKUP_FILE_NAME: &str = "password_backup.csv";

pub mod communications {
    pub const WRONG_KEY: &str = "Wrong key! Try again.";
    pub const ENTER_KEY: &str = "Enter key: ";
    pub const CONFIRM_KEY: &str = "Confirm key: ";
    pub const ERROR_CONFIRMING_KEY: &str = "Your keys do not match!";
}

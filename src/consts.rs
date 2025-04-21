pub const CONF_ACCESS_CHECK: &str = "access_key_hash";
pub const HASH_COST: u32 = 8;
pub const LOWERCASE_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub const NUMBERS: &str = "0123456789";
pub const SPECIAL_CHARACTERS: &str = "!@#$%^&*()-_=+[]{}<>/?";
pub const BACKUP_FILE_NAME: &str = "password_backup.csv";
pub const FIRST_KEY_ASK: &str = "You private key:";
pub const FAIL_KEY_ASK: &str = "Wrong key! Try again:";

pub enum ConfigParams {
    AccessCheck,
}

impl ConfigParams {
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::AccessCheck => "access_key_hash",
        };

        String::from(str)
    }
}

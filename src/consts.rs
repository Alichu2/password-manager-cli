pub const HASH_COST: u32 = 8;
pub const LOWERCASE_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub const NUMBERS: &str = "0123456789";
pub const SPECIAL_CHARACTERS: &str = "!@#$%^&*()-_=+[]{}<>/?";
pub const BACKUP_FILE_NAME: &str = "password_backup.csv";
pub const CSV_PLACE: &str = "place";
pub const CSV_USERNAME: &str = "username";
pub const CSV_PASSWORD: &str = "password";
pub const CSV_ENCRYPTED: &str = "encrypted";

pub mod communications {
    pub const WRONG_KEY: &str = "Wrong key! Try again.";
    pub const ENTER_KEY: &str = "Enter key: ";
    pub const CONFIRM_KEY: &str = "Confirm key: ";
    pub const ERROR_CONFIRMING_KEY: &str = "Your keys do not match!";
    pub const INIT_KEY: &str =
        "Enter a key used to encrypt passwords (if you forget this key, the passwords are lost): ";
    pub const SELECTED_PASSWORD: &str = "Selected password:";
    pub const NEW_PLACE: &str = "New place (leave empty to keep current):";
    pub const NEW_USERNAME: &str = "New username (leave empty to keep current):";
    pub const NEW_PASSWORD: &str = "New password (leave empty to keep current):";
    pub const GENERATED_PASSWORD: &str = "Generated Password:";
    pub const ENTER_PASSWORD: &str = "Enter the password:";
    pub const SAVED_PASSWORD: &str = "Saved password:";
    pub const PASSWORD_DELETE_CONFIRMATION: &str = "Are you sure you want to delete this password?";
    pub const YES_NO: &str = "[y/n]";
    pub const OPERATION_CANCELLED: &str = "Operation cancelled.";
}

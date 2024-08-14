use crate::password_operator::Password;

pub fn display_passwords(passwords: &Vec<Password>) -> String {
    let mut result = String::new();

    for (index, password) in passwords.iter().enumerate() {
        result.push_str(&format!("\n{}:\n{}\n", index, password))
    }

    result
}

pub fn ask_user_for_key() {
    unimplemented!()
}

pub fn create_backup() {
    unimplemented!()
}

pub fn restore_backup() {
    unimplemented!()
}

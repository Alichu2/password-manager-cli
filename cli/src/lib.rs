pub mod cli {
    use std::io::stdin;
    use std::process::exit;
    use std::env::current_dir;
    use std::path::PathBuf;
    use rpassword;

    pub struct  CLI {
        pub arguments: Vec<String>,
    }
    

    impl CLI {
        pub fn from(arguments: Vec<String>) -> Self {
            Self {
                arguments
            }
        }

        pub fn new() -> Self {
            Self {
                arguments: Vec::new()
            }
        }

        pub fn contains_flag(&self, flag: &str) -> bool {
            self.arguments.iter().any(|arg| arg.to_string()=="--".to_string() + flag)
        }

        pub fn get_command(&self) -> &str {
            match self.arguments.get(1) {
                Some(val) => &val,
                None => {
                    println!("No command was found. Use `--help` for more info.");
                    exit(1);
                }
            }
        }

        pub fn get_command_index(&self, index: usize, error: &str) -> &str {
            match self.arguments.get(index + 1) {
                Some(val) => &val,
                None => {
                    println!("{} Use `--help` for more info.", error);
                    exit(1);
                }
            }
        }

        pub fn get_param(&self, param_name: &str) -> String {
            for arg_index in 0..self.arguments.len() {
                if self.arguments[arg_index] == "-".to_string() + param_name {
                    let val = match self.arguments.get(arg_index + 1) {
                        Some(val) => val,
                        None => {
                            println!("No parameter was given for `{}`.", param_name);
                            exit(1);
                        }
                    };
                    return val.to_string();
                }
            }
            return "".to_string()
        }

        pub fn get_current_dir(&self) -> PathBuf {
            match current_dir() {
                Ok(val) => val,
                Err(_) => {
                    println!("Error getting currenct dir.");
                    exit(1);
                }
            }
        }

        pub fn get_password(&self, prompt: &str) -> String {
            match rpassword::prompt_password(prompt) {
                Ok(val) => {
                    if val.trim().is_empty() {
                        println!("No password entered. Try Again!");
                        exit(1);
                    }
                    val.trim().to_string()
                },
                Err(_) => {
                    println!("Error reading password");
                    exit(1);
                }
            }
        }

        pub fn ask(&self, question: &str) -> String {
            let mut awnser = String::new();
            println!("{}", question);
            stdin().read_line(&mut awnser).expect("Failed to read line. Try Again.");
            awnser.trim().to_string()
        }

        pub fn read_required(&self, flag: &str, description: &str) -> String {
            let mut val: String = self.get_param(flag);
            if val.is_empty() {
                val = self.ask(description);
                if val.is_empty() {
                    println!("Please try again by actually entering a value.");
                    std::process::exit(2);
                }
            }
            val
        }

        pub fn help(&self) {
            println!("   Copyright (c) 2023 Aliyu Nauke
====================================

This is the password-manager-cli help guide.

Commands:
    load           Find a previously generated password with a place name, url or ID.
    generate       Generate a password.
    backup         Create a backup of the saved passwords to a file. The contents of the file can be not encrypted with --no-encrypt.
    restore        Load a backup and save all the passwords in the backup file. The passwords can be saved without encryption with --no-encrypt.
    delete         Delete a password with a specified place. If multiple are found, it will prompt to specify the one to be eliminated.
    add            Add a custom password to save that isn't generated.

Arguments:
    --save         Save the generated password. The password will be encrypted.
    --no-special   Exclude spacial characters in the generated password.
    --no-upper     Exclude uppercase characters in the generated password.
    --no-digits    Exclude digits in the generated password.
    --no-encrypt   Won't encrypt your password when saving. It will still prompt you for the access key, but input will be ignored.
    --help         Manual (what you are currently reading).
    --new-key      Enter your key. Can only be done once so remember it as it is necessary to decrypt passwords. The key will not be saved in any form.
    --all          Selects all passwords for loading and displaying.
    --version      Password-Manager's version.

    -u (username)  Password's username.
    -p (name)      Password's name.
    -l (length)    Length of the generated password. Defaults to 6 characters.

If you want to delete all the passwords, you can delete the file data.sqlite in the folder ~/.password-manager/.

Visit <https://github.com/Alichu2/password-manager-cli> for more information.");
        }
    }
}
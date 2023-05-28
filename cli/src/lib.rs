// Library used to access commandline arguments and user input.

pub mod cli {
    use std::io::stdin;
    use std::process::exit;
    use rpassword;

    pub struct  CLI {
        arguments: Vec<String>,
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
            let flag_in_cli = "--".to_string() + flag;
            self.arguments.iter().any(|arg| arg == &flag_in_cli)
        }

        pub fn get_command(&self) -> Option<&String> {
            self.get_argument(0)
        }

        pub fn get_argument(&self, index: usize) -> Option<&String> {
            self.arguments.get(index + 1)
        }

        pub fn find_argument(&self, needle: &str) -> Option<usize> {
            for (index, item) in self.arguments.iter().enumerate() {
                if item == needle {
                    return Some(index - 1);
                }
            }
            None
        }

        pub fn get_option_value(&self, param_name: &str) -> Option<&String> {
            match self.find_argument(param_name) {
                Some(val) => self.get_argument(val + 1),
                None => None,
            }
        }

        pub fn prompt_password(&self, prompt: &str) -> String {
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

        pub fn prompt(&self, question: &str) -> Option<String> {
            let mut answer = String::new();
            println!("{}", question);
            match stdin().read_line(&mut answer) {
                Ok(_) => Some(answer.trim().to_string()),
                Err(_) => None
            }
        }

        pub fn prompt_missing_flag(&self, flag: &str, question: &str) -> Option<String> {
            match self.get_option_value(flag) {
                Some(val) => Some(val.clone()),
                None => self.prompt(question),
            }
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
    delete         Delete a password with a specified place. If multiple are found, it will prompt to specify the one that should be deleted.
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
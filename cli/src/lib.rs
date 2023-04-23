pub mod cli {
    use std::io::stdin;

    pub struct  CLI {
        pub arguments: Vec<String>,
    }
    

    impl CLI {
        pub fn contains_flag(&self, flag: &str) -> bool {
            self.arguments.iter().any(|arg| arg.to_string()=="--".to_string() + flag)
        }

        pub fn get_command(&self) -> &str {
            &self.arguments[1]
        }

        pub fn get_param(&self, param_name: &str) -> String {
            for arg_index in 0..self.arguments.len() {
                if self.arguments[arg_index] == "-".to_string() + param_name {
                    return (self.arguments[arg_index + 1]).to_string();
                }
            }
            return "".to_string()
        }

        pub fn ask(&self, question: &str) -> String {
            let mut awnser = String::new();
            println!("{}", question);
            stdin().read_line(&mut awnser).expect("Failed to read line. Try Again.");
            awnser
        }

        pub fn read_required(&self, flag: &str, description: &str) -> String {
            let mut val: String = self.get_param(flag);
            if val.is_empty() {
                val = self.ask(description);
                if val.trim().is_empty() {
                    println!("Please try again by actually entering a value.");
                    std::process::exit(2);
                }
            }
            val
        }

        pub fn help(&self) {
            println!("This is the CLI Password Manager help guide.
Commands:
    load           Find a previously generated password with a place name, url or ID.
    generate       Generate a password. Doesn't have to be saved.
    backup         Create a backup of the saved passwords to a file. The contents of the file can be not encrypted with --no-encrypt.
    restore        Load a backup and save all the passwords in the backup file. The passwords can be saved without encryption with --no-encrypt.
    delete         Delete a password with a specified place. If multiple are found, it will prompt to specify the one to be eliminated.
    add            Add a custom password to save that isn't generated.

Arguments:
    --save         Save the generated password. The password will be encrypted.
    --no-special   Don't include spacial characters in the generated password.
    --no-upper     Don't include uppercase characters in the generated password.
    --no-digits    Don't include digits in the generated password.
    --no-encrypt   Wont encrypt your password when saving. It will still prompt you for the access key, but input will be ignored.
    --help         Manual (what you are currently reading).
    --new-key      Enter your key. Can only be done once so remember it as it is necessary to decrypt passwords. The key will not be saved in any form.
    --all          Selects all passwords for loading and displaying.
    --version      Password-Manager's version.

    -u (username)  Username for the saved password.
    -p (place)     Place name, url or ID for the usage of the password.
    -l (length)    Length of the generated password. Defaults to 6 characters.
    -f (file)      Specify the file use to restore the backuped passwords.

If you want to delete all the passwords, you can delete the file data.sqlite in the folder ~/.password-manager/.");
        }
    }
}
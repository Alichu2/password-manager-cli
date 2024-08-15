use clap::{Parser, Subcommand};
use password_manager::backups::create_backup;
use password_manager::database::create_new_save_file;
use password_manager::password_operator::{get_all_decrypted_passwords, Password};
use password_manager::security::verify_key;
use rpassword::prompt_password;
use std::env;
use std::io::stdin;
use std::process::exit;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new password.
    Generate {
        /// Lenght of the generated password.
        #[arg(short, long, default_value_t = 12)]
        length: usize,
        /// Weather or not the password should have specail characters.
        #[arg(long)]
        no_special: bool,
        /// Weather or not the password should have upper case characters.
        #[arg(long)]
        no_uppercase: bool,
        /// Weather or not the password should have numbers.
        #[arg(long)]
        no_numbers: bool,
        /// Save the generated password to the database.
        #[arg(short, long, requires_all = ["place", "username"])]
        save: bool,
        /// Password's place.
        #[arg(short, long)]
        place: Option<String>,
        /// Password's username.
        #[arg(short, long)]
        username: Option<String>,
        /// Weather or not to encrypt the password if saved.
        #[arg(short, long)]
        no_encrypt: bool,
    },
    /// Add a new password to the database.
    Add {
        /// Password's place.
        #[arg(short, long)]
        place: String,
        /// Password's username.
        #[arg(short, long)]
        username: String,
        /// Weather or not to encrypt the password if saved.
        #[arg(short, long)]
        no_encrypt: bool,
    },
    /// Delete a password from the database.
    Delete {
        /// Password's place.
        place: String,
        /// Delete without confirmation.
        #[arg(short, long)]
        force: bool,
    },
    /// Load a password from the database.
    #[group(required = true, multiple = false)]
    Load {
        /// Password's place.
        place: Option<String>,
        /// Load all paswords
        #[arg(long)]
        all: bool,
    },
    /// Back the passwords up.
    Backup,
    // /// Restore passwords from a backup.
    // Restore {
    //     /// Restore file.
    //     file: String,
    // },
    /// Initial command to create a database with a key.
    CreateDatabase,
}

#[async_std::main]
async fn main() {
    let cli = Cli::parse();
    println!("Currently working on version 2. It will be better and stronger.");

    match cli.command {
        Commands::Generate {
            save,
            length,
            no_special,
            no_uppercase,
            no_numbers,
            place,
            username,
            no_encrypt,
        } => {
            if !save {
                let new_password =
                    Password::generate_password(length, !no_special, !no_numbers, !no_uppercase);
                println!("Generated password: {}", new_password);
            } else {
                let mut new_password = Password::new(username.unwrap(), place.unwrap(), None);
                new_password
                    .generate_and_attach_password(length, !no_special, !no_numbers, !no_uppercase)
                    .await;

                if !no_encrypt {
                    let key = ask_key().await;

                    println!("Generated Password:\n{}", new_password);

                    new_password.encrypt_password(&key).unwrap();
                } else {
                    println!("Generated Password:\n{}", new_password);
                }

                new_password.save().await.unwrap();
            }
        }
        Commands::Load { place, all } => {
            if all {
                let key = ask_key().await;
                let all_passwords = get_all_decrypted_passwords(&key).await;

                println!("{}", display_passwords(&all_passwords));
            } else {
                let mut loaded_password = Password::from(place.unwrap()).await;

                if loaded_password.is_encrypted() {
                    let key = ask_key().await;

                    loaded_password
                        .decrypt_password(&key)
                        .expect("Error decrypting password.");
                }

                println!("Password:\n{}", loaded_password);
            }
        }
        Commands::Add {
            place,
            username,
            no_encrypt,
        } => {
            let password = ask_question("Enter password you desire to save:\n");
            let mut new_password = Password::new(username, place, Some(password));

            if !no_encrypt {
                let key = ask_key().await;

                println!("Saved password:\n{}", new_password);

                new_password
                    .encrypt_password(&key)
                    .expect("Error encrypting password.");
            } else {
                println!("Saved password:\n{}", new_password);
            }

            new_password.save().await.expect("Error saving password.");
        }
        Commands::Delete { place, force } => {
            let password_in_question = Password::from(place).await;

            if !force {
                println!("Selected password to delete:\n{}", &password_in_question);
                match ask_question("Are you sure you want to delete this password? [y/n]: ")
                    .as_str()
                {
                    "y" => (),
                    "n" => {
                        println!("Aborting");
                        exit(0);
                    }
                    other => {
                        println!("Did not recognize {}. Aborting", other);
                        exit(0);
                    }
                };
            }

            password_in_question.delete().await;
        }
        Commands::Backup => {
            let key = ask_key().await;

            create_backup(&mut env::current_dir().unwrap(), &key).await;
        }
        // Commands::Restore { file } => restore_backup(file),
        Commands::CreateDatabase => {
            create_new_save_file(&prompt_password("Enter a key used to encrypt passwords (if you forget this key, the passwords are lost): ").expect("Error reading your brand new key.")).await;
        }
    }
}

pub fn ask_question(question: &str) -> String {
    let mut answer = String::new();

    print!("{}", question);
    stdin()
        .read_line(&mut answer)
        .expect("Error reading input.");

    answer.trim().to_string()
}

async fn ask_key() -> String {
    let mut key = String::new();

    while !verify_key(&key).await {
        key = prompt_password("Enter your key: ").expect("Error reading secret.");
    }

    key
}

pub fn display_passwords(passwords: &Vec<Password>) -> String {
    let mut result = String::new();

    for (index, password) in passwords.iter().enumerate() {
        result.push_str(&format!("\n{}:\n{}\n", index, password))
    }

    result
}

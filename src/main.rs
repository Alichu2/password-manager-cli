use clap::{Parser, Subcommand};
use password_manager::password_operator::Password;

use std::io::stdin;

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
        /// Should the password have special characters.
        #[arg(long)]
        no_special: bool,
        /// Should the password have upper case characters.
        #[arg(long)]
        no_uppercase: bool,
        /// Should the password have numbers.
        #[arg(long)]
        no_numbers: bool,
        /// Save the generated password to the database.
        #[arg(short, long, requires_all = ["place", "username"])]
        save: bool,
        /// Password place (eg. google) used later to load from database.
        #[arg(short, long)]
        place: Option<String>,
        /// Password username or email.
        #[arg(short, long)]
        username: Option<String>,
        /// Should the password be encrypted if saved.
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
        /// Should the password be encrypted if saved.
        #[arg(short, long)]
        no_encrypt: bool,
    },
    /// Delete a password from the database.
    Delete {
        /// Password's place.
        place: String,
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
            commands::generate(
                save,
                length,
                no_special,
                no_uppercase,
                no_numbers,
                place,
                username,
                no_encrypt,
            )
            .await
        }
        Commands::Load { place, all } => commands::load(place, all).await,
        Commands::Add {
            place,
            username,
            no_encrypt,
        } => commands::add_password(place, username, no_encrypt).await,
        Commands::Delete { place } => commands::delete(place).await,
        Commands::Backup => commands::backup().await,
        // Commands::Restore { file } => restore_backup(file),
        Commands::CreateDatabase => commands::create_database().await,
    }
}

mod commands {
    use password_manager::{
        backups::create_backup,
        database::manager::create_new_save_file,
        objects::input_key::ask_valid_key,
        password_operator::{
            get_all_decrypted_passwords, Password, PasswordBuildOptions, PasswordBuilder,
        },
    };
    use rpassword::prompt_password;
    use std::{
        env,
        io::{stdin, stdout},
    };

    use crate::display_passwords;

    use super::ask_question;

    pub async fn backup() {
        let mut read = stdin().lock();
        let mut write = stdout().lock();

        let key = ask_valid_key(&mut read, &mut write)
            .await
            .expect("Error getting key.");
        let mut current_dir = env::current_dir().unwrap();

        // TODO: Why does current_dir need to be mutuable?
        create_backup(&mut current_dir, &key).await;
    }

    pub async fn create_database() {
        let key = prompt_password("Enter a key used to encrypt passwords (if you forget this key, the passwords are lost): ").expect("Error reading your brand new key.");

        create_new_save_file(&key).await;
    }

    pub async fn generate(
        save: bool,
        length: usize,
        no_special: bool,
        no_uppercase: bool,
        no_numbers: bool,
        place: Option<String>,
        username: Option<String>,
        no_encrypt: bool,
    ) {
        let options = PasswordBuildOptions {
            length,
            use_special: !no_special,
            use_upper: !no_uppercase,
            use_numbers: !no_numbers,
        };

        if !save {
            let new_password = PasswordBuilder::generate_password(options);
            println!("Generated password: {}", new_password);
        } else {
            let password_builder =
                PasswordBuilder::from(username.unwrap(), place.unwrap(), options);
            let mut new_password = password_builder.to_password();

            if !no_encrypt {
                let mut read = stdin().lock();
                let mut write = stdout().lock();

                let key = ask_valid_key(&mut read, &mut write)
                    .await
                    .expect("Error getting key.");

                println!("Generated Password:\n{}", new_password);

                new_password.encrypt_password(&key).unwrap();
            } else {
                println!("Generated Password:\n{}", new_password);
            }

            new_password.save().await.unwrap();
        }
    }

    pub async fn add_password(place: String, username: String, no_encrypt: bool) {
        let password = ask_question("Enter password you desire to save:\n");
        let mut new_password = Password::new(username, place, password);

        if !no_encrypt {
            let mut read = stdin().lock();
            let mut write = stdout().lock();

            let key = ask_valid_key(&mut read, &mut write)
                .await
                .expect("Error getting key.");

            println!("Saved password:\n{}", new_password);

            new_password
                .encrypt_password(&key)
                .expect("Error encrypting password.");
        } else {
            println!("Saved password:\n{}", new_password);
        }

        new_password.save().await.expect("Error saving password.");
    }

    pub async fn load(place: Option<String>, all: bool) {
        let mut read = stdin().lock();
        let mut write = stdout().lock();

        if all {
            let valid_key = ask_valid_key(&mut read, &mut write)
                .await
                .expect("Error getting key.");
            let all_passwords = get_all_decrypted_passwords(&valid_key).await;

            println!("{}", display_passwords(&all_passwords));
        } else {
            let mut loaded_password = Password::from(place.unwrap()).await;

            if loaded_password.is_encrypted() {
                let valid_key = ask_valid_key(&mut read, &mut write)
                    .await
                    .expect("Error getting key.");

                loaded_password
                    .decrypt_password(&valid_key)
                    .expect("Error decrypting password.");
            }

            println!("Password:\n{}", loaded_password);
        }
    }

    pub async fn delete(place: String) {
        let password = Password::from(place).await;

        println!("Selected password to delete:\n{}", &password);
        let confirmation = ask_question("Are you sure you want to delete this password? [y/n]: ");

        match confirmation.as_str() {
            "y" => password.delete().await,
            "n" => {
                println!("Aborting");
            }
            other => {
                println!("Did not recognize {}. Aborting", other);
            }
        };
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

pub fn display_passwords(passwords: &Vec<Password>) -> String {
    let mut result = String::new();

    for (index, password) in passwords.iter().enumerate() {
        result.push_str(&format!("\n{}:\n{}\n", index, password))
    }

    result
}

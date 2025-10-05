use clap::{Parser, Subcommand};
use password_manager::{errors::Error, password_operator::Password};

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
        /// List of characters that should be excluded from the password.
        #[arg(short, long, default_value_t = String::new())]
        exclude: String,
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
    /// Edit an already existing password.
    Edit {
        /// Password's place.
        place: String,
        /// Set whether the updated password should be encrypted or not.
        #[arg(long)]
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
    /// List all the saved places in the database.
    List,
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

    let result = match cli.command {
        Commands::Generate {
            save,
            length,
            no_special,
            no_uppercase,
            no_numbers,
            place,
            username,
            no_encrypt,
            exclude,
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
                exclude,
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
        Commands::List => commands::list().await,
        Commands::Edit { place, no_encrypt } => commands::edit(place, no_encrypt).await,
        // Commands::Restore { file } => restore_backup(file),
        Commands::CreateDatabase => commands::create_database().await,
    };

    pretty_error(result);
}

mod commands {
    use password_manager::{
        backups::create_backup,
        database::utils::{create_new_save_file, get_validated_conn},
        errors::Error,
        password_operator::{
            get_all_decrypted_passwords, Password, PasswordBuildOptions, PasswordBuilder,
        },
        user_functions::ask_valid_key,
    };
    use rpassword::prompt_password;
    use std::env;

    use crate::display_passwords;

    use super::ask_question;

    pub async fn backup() -> Result<(), Error> {
        let mut conn = get_validated_conn().await?;

        let key = ask_valid_key(&mut conn).await.expect("Error getting key.");
        let current_dir = env::current_dir().unwrap();

        create_backup(&current_dir, &key, &mut conn).await?;

        Ok(())
    }

    pub async fn create_database() -> Result<(), Error> {
        let key = prompt_password("Enter a key used to encrypt passwords (if you forget this key, the passwords are lost): ").map_err(|_| Error::ReadError)?;

        create_new_save_file(&key).await?;

        Ok(())
    }

    pub async fn list() -> Result<(), Error> {
        let mut conn = get_validated_conn().await?;

        let places = conn
            .get_all_passwords()
            .await?
            .into_iter()
            .map(|password| password.place)
            .collect::<Vec<_>>();

        println!("{}", places.join("\n"));

        Ok(())
    }

    pub async fn edit(place: String, no_encrypt: bool) -> Result<(), Error> {
        let mut conn = get_validated_conn().await?;
        let key = ask_valid_key(&mut conn).await?;
        let mut password = Password::from(place, &mut conn).await?;

        if password.is_encrypted() {
            password.decrypt_password(&key)?;
        }

        println!("Selected password:\n{}", &password);

        let new_place = ask_question("New place (leave empty to keep current):")?;
        let new_username = ask_question("New username (leave empty to keep current):")?
            .unwrap_or(password.username.clone());
        let new_password = ask_question("New password (leave empty to keep current):")?
            .unwrap_or(password.password.clone());

        if new_place.is_none() {
            password.username = new_username;
            password.password = new_password;

            if no_encrypt && password.is_encrypted() {
                password.decrypt_password(&key)?;
            }
            if !no_encrypt && !password.is_encrypted() {
                password.encrypt_password(&key);
            }

            password.update(&mut conn).await?;
        } else {
            let mut new_password = Password::new(new_username, new_place.unwrap(), new_password);

            if !no_encrypt {
                new_password.encrypt_password(&key);
            }

            new_password.save(&mut conn).await?;
            password.delete(&mut conn).await?;
        }

        Ok(())
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
        exclude: String,
    ) -> Result<(), Error> {
        let options = PasswordBuildOptions {
            length,
            use_special: !no_special,
            use_upper: !no_uppercase,
            use_numbers: !no_numbers,
            exclude_char: exclude.chars().collect::<Vec<_>>(),
        };

        if !save {
            let new_password = PasswordBuilder::generate_password(options);
            println!("Generated password: {}", new_password);
        } else {
            let password_builder =
                PasswordBuilder::from(username.unwrap(), place.unwrap(), options);
            let mut new_password: Password = password_builder.into();
            let mut conn = get_validated_conn().await?;

            if !no_encrypt {
                let key = ask_valid_key(&mut conn).await?;

                println!("Generated Password:\n{}", new_password);

                new_password.encrypt_password(&key);
            } else {
                println!("Generated Password:\n{}", new_password);
            }

            new_password.save(&mut conn).await?;
        }
        Ok(())
    }

    pub async fn add_password(
        place: String,
        username: String,
        no_encrypt: bool,
    ) -> Result<(), Error> {
        let password = ask_question("Enter password you desire to save:")?;

        if password.is_none() {
            return Err(Error::EmptyInput);
        }

        let mut new_password = Password::new(username, place, password.unwrap());
        let mut conn = get_validated_conn().await?;

        if !no_encrypt {
            let key = ask_valid_key(&mut conn).await?;

            println!("Saved password:\n{}", new_password);

            new_password.encrypt_password(&key);
        } else {
            println!("Saved password:\n{}", new_password);
        }

        new_password.save(&mut conn).await?;

        Ok(())
    }

    pub async fn load(place: Option<String>, all: bool) -> Result<(), Error> {
        let mut conn = get_validated_conn().await?;

        if all {
            let valid_key = ask_valid_key(&mut conn).await?;
            let all_passwords = get_all_decrypted_passwords(&valid_key, &mut conn).await?;

            println!("{}", display_passwords(&all_passwords));
        } else {
            let mut loaded_password = Password::from(place.unwrap(), &mut conn).await?;

            if loaded_password.is_encrypted() {
                let valid_key = ask_valid_key(&mut conn).await?;

                loaded_password.decrypt_password(&valid_key)?;
            }

            println!("Password:\n{}", loaded_password);
        }

        Ok(())
    }

    pub async fn delete(place: String) -> Result<(), Error> {
        let mut conn = get_validated_conn().await?;
        let password = Password::from(place, &mut conn).await?;

        println!("Selected password:\n{}", &password);
        let confirmation = ask_question("Are you sure you want to delete this password? [y/n]: ")?;

        match confirmation.as_deref() {
            Some("y") => password.delete(&mut conn).await?,
            Some("n") => {
                println!("Operation cancelled.");
            }
            Some(other) => {
                println!("Did not recognize {}. Aborting", other);
            }
            None => {
                println!("Operation cancelled")
            }
        };

        Ok(())
    }
}

pub fn ask_question(question: &str) -> Result<Option<String>, Error> {
    let mut answer = String::new();

    println!("\n{}", question);
    stdin()
        .read_line(&mut answer)
        .map_err(|_| Error::ReadError)?;
    let trimmed_answer = answer.trim().to_string();

    if trimmed_answer.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed_answer))
    }
}

pub fn display_passwords(passwords: &Vec<Password>) -> String {
    let mut result = String::new();

    for (index, password) in passwords.iter().enumerate() {
        result.push_str(&format!("\n{}:\n{}\n", index, password))
    }

    result
}

pub fn pretty_error(result: Result<(), Error>) {
    match result {
        Ok(_) => (),
        Err(err) => println!("Error: {err}"),
    }
}

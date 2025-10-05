use clap::{Parser, Subcommand};
use password_manager::{operations, utils::pretty_error};

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
    /// Back the passwords into a CSV file. The passwords are all decrypted.
    Backup,
    /// Similar to backup, but it just dumps the database contents into a CSV without encrypting or decrypting. Useful for automatic periodic backups.
    DumpDatabase,
    /// List all the saved places in the database.
    List,
    /// Restore passwords from a database dump.
    LoadDump {
        /// Database dump file.
        file: String,
    },
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
            operations::generate(
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
        Commands::Load { place, all } => operations::load(place, all).await,
        Commands::Add {
            place,
            username,
            no_encrypt,
        } => operations::add_password(place, username, no_encrypt).await,
        Commands::Delete { place } => operations::delete(place).await,
        Commands::Backup => operations::backup().await,
        Commands::List => operations::list().await,
        Commands::DumpDatabase => operations::dump_db().await,
        Commands::Edit { place, no_encrypt } => operations::edit(place, no_encrypt).await,
        Commands::LoadDump { file } => operations::load_dump(file).await,
        Commands::CreateDatabase => operations::create_database().await,
    };

    pretty_error(result);
}

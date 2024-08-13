use std::env;
use std::path::PathBuf;
use std::process::exit;

use password_manager::cli::cli::CLI;
use password_manager::interface::PasswordManagerInterface;

fn main() {
    let cli: CLI = CLI::from(env::args().collect());
    let password_manager: PasswordManagerInterface = PasswordManagerInterface::new();

    if cli.contains_flag("help") {
        cli.help();
    } else if cli.contains_flag("version") {
        println!(
            "password-manager-cli Version: {}\nCopyright (c) 2023 Aliyu Nauke",
            env!("CARGO_PKG_VERSION")
        );
    } else if cli.contains_flag("new-key") {
        let new_key = cli.prompt_loop_password(
            "Please input an access key. This will be used to encrypt and decrypt passwords: ",
        );

        if new_key == cli.prompt_loop_password("Confirm key: ") {
            password_manager.pw_core.create_new_save_file(&new_key);
        }
    } else if !password_manager.pw_core.save_file_exists() {
        println!("Please configure a key. Use `--new-key` or `--help` for more information.");
    } else {
        let command = match cli.get_command() {
            Some(val) => val.as_str(),
            None => exit(1),
        };

        // TODO: Go over all errors in project to see if can be improved.
        match command {
            "generate" => {
                let save = cli.contains_flag("save");
                let length = match cli.get_option_value("-l") {
                    Some(val) => match val.parse() {
                        Ok(val) => val,
                        Err(_) => 6,
                    }
                    None => 6
                };
                let generated_password = password_manager.pw_core.generate_password(length, !cli.contains_flag("no-upper"), !cli.contains_flag("no-digits"), !cli.contains_flag("no-special"));

                if save {
                    password_manager.save_password(
                        generated_password,
                        cli.prompt_loop_missing_flag("-u", "Username for the password:"),
                        cli.prompt_loop_missing_flag("-p", "Name for the password:"),
                        !cli.contains_flag("no-encrypt")
                    );
                }
                else {
                    println!("generated password = {}", generated_password);
                }
            },
            // TODO: Ask user before replacing password.
            "load" => {
                if cli.contains_flag("all") {
                    password_manager.load_all_passwords()
                }
                else {
                    password_manager.load_password(&cli.prompt_loop_missing_flag("-p", "Password Name:"));
                }
            },
            "add" => {
                password_manager.add_password(
                    &cli.prompt("New password: ").unwrap(),
                    &cli.prompt_loop_missing_flag("-u", "Password username:"),
                    &cli.prompt_loop_missing_flag("-p", "Password name:"),
                    !cli.contains_flag("no-encrypt"),
                );
            },
            "delete" => {
                let place = cli.prompt_missing_flag("-p", "Name of password to be deleted:").unwrap();

                password_manager.delete_password(place);
            },
            "backup" => {
                password_manager.create_backup(
                    env::current_dir().unwrap(),
                    !cli.contains_flag("no-encrypt")
                );

                println!("Backup created");
            },
            "restore" => {
                password_manager.restore_backup(
                    PathBuf::from(cli.get_argument(1).unwrap()),
                    !cli.contains_flag("no-encrypt")
                );
            },
            invalid => println!("`{}` is not a recognized command. Please enter a valid command. Use `--help` for more information.", invalid)
        }
    }
}

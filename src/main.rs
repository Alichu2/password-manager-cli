mod interface;

use std::env;
use std::path::PathBuf;
use std::process::exit;

use cli::cli::CLI;
use interface::PasswordManagerInterface;


fn main() {
    let cli: CLI = CLI::from(env::args().collect());
    let interface: PasswordManagerInterface = PasswordManagerInterface::new();

    if cli.contains_flag("help") {
        cli.help();
    }
    else if cli.contains_flag("version") {
        println!("password-manager-cli Version: {}\nCopyright (c) 2023 Aliyu Nauke", env!("CARGO_PKG_VERSION"));
    }
    else if cli.contains_flag("new-key") {
        let new_key = cli.prompt_password("Please input an access key. This will be used to encrypt and decrypt passwords. Keys are never stored: ");

        if new_key == cli.prompt_password("Confirm key: ") {
            interface.pw_core.create_new_save_file(&new_key);
        }
    }
    else if !interface.pw_core.save_file_exists() {
        println!("Please configure a key. Use `--new-key` or `--help` for more information.");
    }
    else {
        let command = match cli.get_command() {
            Some(val) => val.as_str(),
            None => exit(1)
        };

        // TODO: Smarten errors. Possible loop until value entered.
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
                let generated_password = interface.pw_core.generate_password(length, !cli.contains_flag("no-upper"), !cli.contains_flag("no-digits"), !cli.contains_flag("no-special"));

                if save {
                    interface.generate_and_save(
                        generated_password,
                        cli.prompt_missing_flag("-u", "Username for the password:").unwrap(),
                        cli.prompt_missing_flag("-p", "Name for the password:").unwrap(),
                        !cli.contains_flag("no-encrypt")
                    );
                }
                else {
                    println!("generated password = {}", generated_password);
                }
            },
            "load" => {
                if cli.contains_flag("all") {
                    interface.load_all_passwords()
                }
                else {
                    interface.load_password(&cli.prompt_missing_flag("-p", "Password Name:").unwrap());
                }
            },
            "add" => {
                interface.add_password(
                    &cli.prompt("New password: ").unwrap(),
                    &cli.prompt_missing_flag("-u", "Password username: ").unwrap(),
                    &cli.prompt_missing_flag("-p", "Password name: ").unwrap(),
                    !cli.contains_flag("no-encrypt"),
                );
            },
            "delete" => {
                let place = cli.prompt_missing_flag("-p", "Name of password to be deleted:").unwrap();

                interface.delete_password(place);
            },
            "backup" => {
                interface.create_backup(
                    env::current_dir().unwrap(),
                    !cli.contains_flag("no-encrypt")
                );

                println!("Backup created");
            },
            "restore" => {
                interface.restore_backup(
                    PathBuf::from(cli.get_argument(1).unwrap()),
                    !cli.contains_flag("no-encrypt")
                );
            },
            invalid => println!("`{}` is not a recognized command. Please enter a valid command. Use `--help` for more information.", invalid)
        }
    }
}

mod interface;

use std::env;

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
        let new_key = cli.get_password("Please input an access key. This will be used to encrypt and decrypt passwords. Keys are never stored: ");

        if new_key == cli.get_password("Confirm key: ") {
            interface.create_save_file(&new_key);
        }
    }
    else if !interface.save_file_exists() {
        println!("Please configure a key. Use `--new-key` or `--help` for more information.");
    }
    else {
        match cli.get_command() {
            "generate" => {
                let save = cli.contains_flag("save");
                let length = match cli.get_param("l").parse() {
                    Ok(val) => val,
                    Err(_) => 6
                };

                if save {
                    interface.generate_and_save(
                        !cli.contains_flag("no-special"),
                        !cli.contains_flag("no-upper"),
                        !cli.contains_flag("no-digits"),
                        length,
                        cli.read_required("u", "Username for the password:"),
                        cli.read_required("p", "Name for the password:"),
                        !cli.contains_flag("no-encrypt")
                    );
                }
                else {
                    let generated_password = interface.generate_password(
                        !cli.contains_flag("no-special"),
                        !cli.contains_flag("no-upper"),
                        !cli.contains_flag("no-digits"),
                        length,
                    );

                    println!("generated password = {}", generated_password);
                }
            },
            "load" => {
                if cli.contains_flag("all") {
                    interface.load_all_passwords()
                }
                else {
                    interface.load_password(&cli.read_required("p", "Password Name:"));
                }
            },
            "add" => {
                interface.add_password(
                    &cli.ask("New password: "),
                    &cli.read_required("u", "Password username: "),
                    &cli.read_required("p", "Password name: "),
                    !cli.contains_flag("no-encrypt"),
                );
            },
            "delete" => {
                let place = cli.read_required("p", "Name of password to be deleted:");

                interface.delete_password(place);
            },
            "backup" => {
                interface.create_backup(
                    cli.get_current_dir(),
                    !cli.contains_flag("no-encrypt")
                );

                println!("Backup created");
            },
            "restore" => {
                interface.restore_backup(
                    cli.get_current_dir().join(cli.get_command_index(1, "Please enter file path.")),
                    !cli.contains_flag("no-encrypt")
                );
            },
            invalid => println!("`{}` is not a recognized command. Please enter a valid command. Use `--help` for more information.", invalid)
        }
    }
}

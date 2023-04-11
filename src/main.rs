use std::env;
use rpassword;
use std::fs::File;
use std::io::prelude::*;

mod cli;
mod password;


fn print_passwords(passwords: Vec<[String; 5]>) {
    let num_passwords = passwords.len();
    let key = password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string());

    for password_index in 0..num_passwords {
        println!("\n{}:", password_index);
        println!("  username = {}", passwords[password_index][1]);
        println!("  place = {}", passwords[password_index][2]);
        if passwords[password_index][4] == "1" {
            println!("  password = {}", password::decrypt(&passwords[password_index][0], &key));
        }
        else {
            println!("  password = {}", passwords[password_index][0]);
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if cli::contains_flag(&args, "help") {
        println!("{}", cli::help());
    }
    else if cli::contains_flag(&args, "version") {
        println!("Password-Manager Version: {}", env!("CARGO_PKG_VERSION"));
    }
    else if cli::contains_flag(&args, "new-key") {
        if password::save_file_exists() {
            println!("Access key already exists. Cannot replace.");
        }
        else {
            let new_key = rpassword::prompt_password("Plase input an access key. This will be used to encrypt and decrypt passwords: ").unwrap().trim().to_string();

            if new_key.is_empty() {
                println!("Not a valid key.");
            }
            else if new_key == rpassword::prompt_password("Re-enter: ").unwrap().trim().to_string() {
                password::create_database_tables();
                
                password::save_key(new_key);

                println!("Key saved.");
            }
        }
    }
    else if !password::save_file_exists() {
        println!("You need to configure an encryption key. Use --new-key or --help for more information.");
    }
    else {
        match cli::get_command(&args) {
            "generate" => {
                let length = match cli::get_param(&args, "l").parse() {
                    Ok(num_chars) => num_chars,
                    Err(_) => 6
                };
    
                let password = password::generate_password(
                    length,
                    !cli::contains_flag(&args, "no-upper"),
                    !cli::contains_flag(&args, "no-digits"),
                    !cli::contains_flag(&args, "no-special"),
                );

                if cli::contains_flag(&args, "save") {
                    let place = cli::read_required("p", "\nEnter an place name, url or ID for the password:", &args);
                    let username = cli::read_required("u", "\nEnter an username for the password:", &args);

                    if !cli::contains_flag(&args, "no-encrypt") {
                        let key = password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string());
                        let encrypted_pass = password::encrypt(&password, &key);
                        password::save_password(&encrypted_pass, &username, &place, true);
                    }
                    else {
                        password::save_password(&password, &username, &place, false);
                    }

                    println!("\nYour new password and username for {}:\nPassword: {}\nUsername: {}", place.trim(), password, username.trim());
                }
                else {
                    println!("Generated Password: {}", password);
                }
            },
            "load" => {
                let password_data;
                if cli::contains_flag(&args, "all") {
                    password_data = password::get_all_passwords();
                }
                else {
                    let place = cli::read_required("p", "\nEnter the place name, url or ID for the password:", &args);
                    password_data = password::find_password(&place);
                }

                println!("{} password(s) found.", password_data.len());

                if !(password_data.len() == 0) {
                    print_passwords(password_data);
                }
            },
            "delete" => {
                let place = cli::read_required("p", "\nEnter the place name, url or ID for the password:", &args);
                
                let password_data = password::find_password(&place);
                
                if password_data.len() > 1 {
                    println!("Multiple passwords have been found with the same place. Please select the on you want to delete.");

                    let mut ids: Vec<usize> = Vec::new();

                    for password in password_data.iter() {
                        ids.push(password[3].parse().unwrap());
                    }
                    
                    print_passwords(password_data);
                    
                    let eliminate = cli::ask("\nEnter the password number:").trim().to_string();

                    if cli::ask("\nAre you sure you want to delete? [y/n]:").trim() == "y" {
                        password::delete_password(&ids[eliminate.parse::<usize>().unwrap()].to_string());
                        println!("Deleted password.");
                    }
                }
                else if password_data.len() > 0 {
                    password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string());
                    if cli::ask("\nAre you sure you want to delete? [y/n]:").trim() == "y" {
                        password::delete_password(&password_data[0][3]);
                        println!("Deleted password.");
                    }
                }
                else {
                    println!("No passwords found under that place name.");
                }
            },
            "add" => {
                let place = cli::read_required("p", "\nEnter the place name, url or ID for the password:", &args);
                let password = cli::ask("\nEnter the password:");
                let username = cli::read_required("u", "\nEnter an username for the password:", &args);
                let encrypt = !cli::contains_flag(&args, "no-encrypt");

                if encrypt {
                    let encrypted = password::encrypt(&password, &password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string()));
                    password::save_password(&encrypted, &username, &place, true);
                }
                else {
                    password::save_password(&password, &username, &place, false);
                }

                println!("\nYour new password and username for {}:\nPassword: {}\nUsername: {}", place.trim(), password.trim(), username.trim());
            },
            "backup" => {
                let key = password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string());
                let password_data = password::get_all_passwords();
                let mut file_str = String::new();
                
                for password in password_data.iter() {
                    file_str += &(password::decrypt(&password[0], &key) + "|" + &password[1] + "|" + &password[2] + "\n");
                }
                if !cli::contains_flag(&args, "no-encrypt") {
                    let file_key = rpassword::prompt_password("\nCreate file key (needed later to restore): ").unwrap().trim().to_string();
                    if file_key == rpassword::prompt_password("Re-enter: ").unwrap().trim().to_string() {
                        file_str = password::encrypt(&file_str, &file_key);
                        file_str += "\nUse password-manager CLI with the file key.\n";
                    }
                    else {
                        panic!("Keys don't match.");
                    }
                }
                let mut backup_file = File::create(env::current_dir().unwrap().display().to_string() + "password_backup.txt").unwrap();
                backup_file.write_all(file_str.as_bytes()).unwrap();
            },
            "restore" => {
                let backup_file_loc = env::current_dir().unwrap().display().to_string() + &cli::read_required("f", "Backup file location:", &args);
                let mut backup_file = File::open(&backup_file_loc).unwrap();
                let mut contents = String::new();
                
                backup_file.read_to_string(&mut contents).unwrap();
                
                if contents.contains("\nUse password-manager CLI with the file key.\n") {
                    contents = contents.replace("\nUse password-manager CLI with the file key.\n", "");
                    let file_key = rpassword::prompt_password("File Key: ").unwrap().trim().to_string();
                    
                    contents = password::decrypt(&contents, &file_key);
                }
                
                let mut new_passwords: Vec<Vec<String>> = Vec::new();
                
                for row in contents.split("\n") {
                    if !row.is_empty() {
                        new_passwords.push(row.to_string().split("|").map(String::from).collect());
                    }
                }
                
                let key = password::verify_key(rpassword::prompt_password("\nEnter your access key: ").unwrap().trim().to_string());
                
                if cli::contains_flag(&args, "no-encrypt") {
                    for password in new_passwords.iter() {
                        password::save_password(&password[1], &password[1], &password[2], false);
                    }
                }
                else {
                    for password in new_passwords.iter() {
                        let encrypted_pass = password::encrypt(&password[0], &key);
                        password::save_password(&encrypted_pass, &password[1], &password[2], true);
                    }
                }

                println!("New passwords added.");
            },
            "locate" => {
                println!("{}", env::current_dir().unwrap().display());
            },
            _ => { println!("Please give valid command (use --help for more info)."); }
        }
    }    
}

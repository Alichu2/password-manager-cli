use crate::{
    consts::{
        communications::{
            ENTER_PASSWORD, GENERATED_PASSWORD, INIT_KEY, NEW_PASSWORD, NEW_PLACE, NEW_USERNAME,
            OPERATION_CANCELLED, PASSWORD_DELETE_CONFIRMATION, SAVED_PASSWORD, SELECTED_PASSWORD,
        },
        CSV_ENCRYPTED, CSV_PASSWORD, CSV_PLACE, CSV_USERNAME,
    },
    database::utils::{create_new_save_file, get_validated_conn},
    errors::Error,
    password::{Password, PasswordBuildOptions, PasswordBuilder},
    utils::{ask_bool, ask_question, ask_valid_key, create_backup, find_clomun_index},
};
use rpassword::prompt_password;
use std::{env, fs, io::Read};

pub async fn backup() -> Result<(), Error> {
    let mut conn = get_validated_conn().await?;

    let key = ask_valid_key(&mut conn).await?;
    let current_dir = env::current_dir().map_err(|_| Error::BadDir)?;
    let mut passwords = conn.get_all_passwords().await?;

    for password in passwords.iter_mut() {
        if password.is_encrypted() {
            password.decrypt_password(&key)?;
        }
    }

    create_backup(&current_dir, &passwords)?;

    Ok(())
}

pub async fn dump_db() -> Result<(), Error> {
    let mut conn = get_validated_conn().await?;
    let passwords = conn.get_all_passwords().await?;

    println!(
        "{},{},{},{}",
        CSV_PLACE, CSV_USERNAME, CSV_PASSWORD, CSV_ENCRYPTED
    );

    for password in passwords {
        println!("{}", password.dump())
    }

    Ok(())
}

pub async fn load_dump(file: String) -> Result<(), Error> {
    let mut conn = get_validated_conn().await?;
    let mut contents = String::new();
    let current_path = env::current_dir().map_err(|_| Error::BadDir)?;
    let mut file = fs::File::open(current_path.join(file)).map_err(|_| Error::ReadError)?;

    file.read_to_string(&mut contents)
        .map_err(|_| Error::ReadError)?;

    let mut lines = contents.trim().split("\n");
    let header = lines.next();

    let header_parts = header.ok_or(Error::BadDump)?.split(",");

    if header_parts.clone().count() < 4 {
        return Err(Error::BadHeaders);
    }

    let place_index = find_clomun_index(
        CSV_PLACE,
        "Enter `place` column name:",
        header_parts.clone(),
    )?;
    let username_index = find_clomun_index(
        CSV_USERNAME,
        "Enter `username` column name:",
        header_parts.clone(),
    )?;
    let password_index = find_clomun_index(
        CSV_PASSWORD,
        "Enter `password` column name:",
        header_parts.clone(),
    )?;
    let encrypted_index = find_clomun_index(
        CSV_ENCRYPTED,
        "Enter `encrypted` column name:",
        header_parts,
    )?;

    let mut passwords = Vec::new();

    for (index, line) in lines.enumerate() {
        let parts = line.split(",").map(|v| v.to_owned()).collect::<Vec<_>>();

        let place = parts
            .get(place_index)
            .ok_or(Error::MissingField(CSV_PLACE, index + 2))?
            .to_owned();
        let username = parts
            .get(username_index)
            .ok_or(Error::MissingField(CSV_ENCRYPTED, index + 2))?
            .to_owned();
        let password = parts
            .get(password_index)
            .ok_or(Error::MissingField(CSV_PASSWORD, index + 2))?
            .to_owned();
        let encrypted = parts
            .get(encrypted_index)
            .ok_or(Error::MissingField(CSV_ENCRYPTED, index + 2))?;

        let new_password = Password {
            place,
            username,
            password,
            encrypted: encrypted.parse::<i32>().map_err(|_| Error::ParsingError)?,
        };

        passwords.push(new_password);
    }

    for password in passwords.iter() {
        conn.insert_password(password).await?;
    }

    println!("Loaded {} passwords.", passwords.len());

    Ok(())
}

pub async fn create_database() -> Result<(), Error> {
    let key = prompt_password(INIT_KEY).map_err(|_| Error::ReadError)?;

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

    println!("{}\n{}", SELECTED_PASSWORD, &password);

    let new_place = ask_question(NEW_PLACE)?;
    let new_username = ask_question(NEW_USERNAME)?.unwrap_or(password.username.clone());
    let new_password = ask_question(NEW_PASSWORD)?.unwrap_or(password.password.clone());

    if new_place.is_none() {
        password.username = new_username;
        password.password = new_password;

        if no_encrypt && password.is_encrypted() {
            password.decrypt_password(&key)?;
        }
        if !no_encrypt && !password.is_encrypted() {
            password.encrypt_password(&key);
        }

        conn.update_password(&password).await?;
    } else {
        let mut new_password = Password::new(new_username, new_place.unwrap(), new_password);

        if !no_encrypt {
            new_password.encrypt_password(&key);
        }

        conn.insert_password(&new_password).await?;
        conn.delete_password(&password.place).await?;
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
        println!("{} {}", GENERATED_PASSWORD, new_password);
    } else {
        let password_builder = PasswordBuilder::from(username.unwrap(), place.unwrap(), options);
        let mut new_password: Password = password_builder.into();
        let mut conn = get_validated_conn().await?;

        if !no_encrypt {
            let key = ask_valid_key(&mut conn).await?;

            println!("{}\n{}", GENERATED_PASSWORD, new_password);

            new_password.encrypt_password(&key);
        } else {
            println!("{}\n{}", GENERATED_PASSWORD, new_password);
        }

        conn.insert_password(&new_password).await?;
    }
    Ok(())
}

pub async fn add_password(place: String, username: String, no_encrypt: bool) -> Result<(), Error> {
    let password = ask_question(ENTER_PASSWORD)?;

    if password.is_none() {
        return Err(Error::EmptyInput);
    }

    let mut new_password = Password::new(username, place, password.unwrap());
    let mut conn = get_validated_conn().await?;

    if !no_encrypt {
        let key = ask_valid_key(&mut conn).await?;

        println!("{}\n{}", SAVED_PASSWORD, new_password);

        new_password.encrypt_password(&key);
    } else {
        println!("{}\n{}", SAVED_PASSWORD, new_password);
    }

    conn.insert_password(&new_password).await?;

    Ok(())
}

pub async fn load(place: Option<String>, all: bool) -> Result<(), Error> {
    let mut conn = get_validated_conn().await?;

    if all {
        let valid_key = ask_valid_key(&mut conn).await?;
        let all_passwords = conn.get_all_passwords().await?;

        for (index, mut password) in all_passwords.into_iter().enumerate() {
            if password.is_encrypted() {
                password.decrypt_password(&valid_key)?;
            }

            println!("{}:\n{}", index, password)
        }
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

    println!("{}\n{}", SELECTED_PASSWORD, &password);
    let confirmation = ask_bool(PASSWORD_DELETE_CONFIRMATION)?;

    if confirmation {
        conn.delete_password(&password.place).await?;
    } else {
        println!("{}", OPERATION_CANCELLED);
    }

    Ok(())
}

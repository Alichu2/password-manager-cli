use crate::database::objects::ConfigParams;
use crate::database::queries::DatabaseInterface;
use crate::errors::Error;
use crate::security::save_new_key;
use sqlx::{Connection, SqliteConnection};
use std::fs;
use std::path::PathBuf;

#[allow(unused_imports)]
use dirs::home_dir;
#[allow(unused_imports)]
use std::process::exit;

#[cfg(not(debug_assertions))]
fn get_home_path() -> Result<PathBuf, Error> {
    home_dir().ok_or(Error::NoHomeDir)
}

#[cfg(debug_assertions)]
fn get_home_path() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("./"))
}

pub fn get_save_dir_path() -> Result<PathBuf, Error> {
    Ok(get_home_path()?.join(".password-manager/"))
}

fn get_save_file_path() -> Result<PathBuf, Error> {
    Ok(get_save_dir_path()?.join("data.sqlite"))
}

pub async fn get_sqlite_connection() -> Result<SqliteConnection, Error> {
    SqliteConnection::connect(&(get_save_file_path()?.display().to_string() + "?mode=rwc"))
        .await
        .map_err(|err| Error::DatabaseError(err))
}

pub async fn create_new_save_file(new_key: &str) -> Result<(), Error> {
    if !get_save_file_path()?.exists() {
        let path = get_save_dir_path()?.display().to_string();
        fs::create_dir_all(path).map_err(|_| Error::DirError)?;

        let mut conn = DatabaseInterface::new().await?;

        conn.create_config_table().await?;
        conn.create_password_table().await?;

        save_new_key(new_key, &mut conn).await?;

        Ok(())
    } else {
        Err(Error::SaveFileExists)
    }
}

pub async fn create_save_file() -> Result<(), Error> {
    let mut conn = DatabaseInterface::new().await?;

    conn.create_config_table().await?;
    conn.create_password_table().await?;

    Ok(())
}

pub async fn has_correct_tables(conn: &mut DatabaseInterface) -> Result<bool, Error> {
    let tables = conn.list_tables().await?;

    for table in tables {
        if !(&table == "passwords" || &table == "config") {
            return Err(Error::UnexpectedTable(table));
        }
    }

    Ok(true)
}

pub async fn has_key(conn: &mut DatabaseInterface) -> Result<bool, Error> {
    conn.has_setting(ConfigParams::AccessCheck).await
}

pub async fn get_validated_conn() -> Result<DatabaseInterface, Error> {
    if get_save_file_path()?.exists() {
        let mut conn = DatabaseInterface::new().await?;

        if has_correct_tables(&mut conn).await? && has_key(&mut conn).await? {
            return Ok(conn);
        }
    }
    Err(Error::MissingDatabase)
}

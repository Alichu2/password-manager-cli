use crate::security::save_new_key;
use anyhow::Result;
use sqlx::{Connection, Executor, SqliteConnection};
use std::fs;
use std::path::PathBuf;

#[cfg(not(debug_assertions))]
fn get_home_path() -> PathBuf {
    match home_dir() {
        None => {
            println!("Error: Home Directory Path Not Found.");
            exit(1);
        }
        Some(path) => path,
    }
}

#[cfg(debug_assertions)]
fn get_home_path() -> PathBuf {
    PathBuf::from("./")
}

pub fn get_save_dir_path() -> PathBuf {
    get_home_path().join(".password-manager/")
}

fn get_save_file_path() -> PathBuf {
    get_save_dir_path().join("data.sqlite")
}

fn get_save_file_path_str() -> String {
    get_save_file_path().display().to_string()
}

pub async fn get_sqlite_connection() -> SqliteConnection {
    SqliteConnection::connect(&(get_save_file_path().display().to_string() + "?mode=rwc"))
        .await
        .expect("Error establishing connection to database.")
}

pub fn create_new_save_file(new_key: &str) {
    if !get_save_file_path().exists() {
        create_save_file();
        save_new_key(new_key.to_string());
    } else {
        println!("Save file and key already exists. Cannot regenerate.");
    }
}

pub async fn create_save_file() -> Result<()> {
    fs::create_dir_all(get_save_dir_path().display().to_string())?;

    let mut db_conn = get_sqlite_connection().await;

    db_conn.execute(sqlx::query("CREATE TABLE passwords (password TEXT, username TEXT, place TEXT PRIMARY KEY, is_encrypted NUMBER);")).await?;

    db_conn
        .execute(sqlx::query(
            "CREATE TABLE config (name TEXT PRIMARY KEY, value TEXT);",
        ))
        .await?;

    Ok(())
}

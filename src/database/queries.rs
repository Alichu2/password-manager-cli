use sqlx::{self, SqliteConnection};

use crate::{
    database::objects::{ConfigItem, ConfigParams},
    errors::Error,
    password_operator::Password,
};

use super::utils::get_sqlite_connection;

pub struct DatabaseInterface {
    connection: SqliteConnection,
}

impl DatabaseInterface {
    pub async fn new() -> Result<Self, Error> {
        let connection = get_sqlite_connection().await?;

        Ok(Self { connection })
    }

    pub fn from(connection: SqliteConnection) -> Self {
        Self { connection }
    }

    pub async fn get_setting(&mut self, setting: ConfigParams) -> Result<ConfigItem, Error> {
        sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
            .bind(setting)
            .fetch_one(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))
    }

    pub async fn set_setting(&mut self, setting: ConfigItem) -> Result<(), Error> {
        sqlx::query("INSERT IF NOT EXISTS INTO config (name, value) VALUES (?, ?);")
            .bind(setting.name)
            .bind(setting.value)
            .execute(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))?;

        Ok(())
    }

    pub async fn create_password_table(&mut self) -> Result<(), Error> {
        sqlx::query("CREATE TABLE passwords (password TEXT, username TEXT, place TEXT PRIMARY KEY, encrypted NUMBER);")
            .execute(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))?;

        Ok(())
    }

    pub async fn create_config_table(&mut self) -> Result<(), Error> {
        sqlx::query("CREATE TABLE config (name TEXT PRIMARY KEY, value TEXT);")
            .execute(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))?;

        Ok(())
    }

    pub async fn get_password(&mut self, place: &str) -> Result<Vec<Password>, Error> {
        sqlx::query_as::<_, Password>("SELECT * FROM passwords WHERE place = ?;")
            .bind(place)
            .fetch_all(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))
    }

    pub async fn delete_password(&mut self, place: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM passwords WHERE place = ?;")
            .bind(place)
            .execute(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))?;

        Ok(())
    }

    pub async fn insert_password(&mut self, password: &Password) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO passwords (place, password, username, encrypted) VALUES (?, ?, ?, ?);",
        )
        .bind(&password.place)
        .bind(&password.password)
        .bind(&password.username)
        .bind(password.encrypted)
        .execute(&mut self.connection)
        .await
        .map_err(|err| Error::DatabaseError(err))?;

        Ok(())
    }

    pub async fn get_all_passwords(&mut self) -> Result<Vec<Password>, Error> {
        sqlx::query_as("SELECT * FROM passwords;")
            .fetch_all(&mut self.connection)
            .await
            .map_err(|err| Error::DatabaseError(err))
    }
}

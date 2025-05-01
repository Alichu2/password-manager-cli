use anyhow::Result;
use sqlx::{self, SqliteConnection};

use crate::objects::query_results::{ConfigItem, ConfigParams};

use super::manager::get_sqlite_connection;

pub struct DatabaseInterface {
    connection: SqliteConnection,
}

impl DatabaseInterface {
    pub async fn new() -> Self {
        let connection = get_sqlite_connection().await;

        Self { connection }
    }

    pub fn from(connection: SqliteConnection) -> Self {
        Self { connection }
    }

    pub async fn get_setting(&mut self, setting: ConfigParams) -> Result<ConfigItem> {
        let result = sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
            .bind(setting)
            .fetch_one(&mut self.connection)
            .await?;

        Ok(result)
    }

    pub async fn set_setting(&mut self, setting: ConfigItem) -> Result<()> {
        sqlx::query("INSERT IF NOT EXISTS INTO config (name, value) VALUES (?, ?);")
            .bind(setting.name)
            .bind(setting.value)
            .execute(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn force_set_setting(&mut self, setting: ConfigItem) -> Result<()> {
        sqlx::query("INSERT INTO config (name, value) VALUES (?, ?);")
            .bind(setting.name)
            .bind(setting.value)
            .execute(&mut self.connection)
            .await?;

        Ok(())
    }
}

use anyhow::Result;
use sqlx::{self, SqliteConnection};

use crate::objects::query_results::{ConfigItem, ConfigParams};

use super::manager::get_sqlite_connection;

// TODO: Add connection as a parameter.
pub async fn get_setting(setting: ConfigParams) -> Result<ConfigItem> {
    let result = sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
        .bind(setting)
        .fetch_one(&mut get_sqlite_connection().await)
        .await?;

    Ok(result)
}

pub async fn set_setting(setting: ConfigItem, conn: &mut SqliteConnection) -> Result<()> {
    sqlx::query("INSERT IF NOT EXISTS INTO config (name, value) VALUES (?, ?);")
        .bind(setting.name)
        .bind(setting.value)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn force_set_setting(setting: ConfigItem, conn: &mut SqliteConnection) -> Result<()> {
    sqlx::query("INSERT INTO config (name, value) VALUES (?, ?);")
        .bind(setting.name)
        .bind(setting.value)
        .execute(conn)
        .await?;

    Ok(())
}

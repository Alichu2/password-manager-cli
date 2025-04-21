use anyhow::Result;
use sqlx;

use crate::objects::query_results::{ConfigItem, ConfigParams};

use super::manager::get_sqlite_connection;

pub async fn get_setting(setting: ConfigParams) -> Result<ConfigItem> {
    let result = sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
        .bind(setting)
        .fetch_one(&mut get_sqlite_connection().await)
        .await?;

    Ok(result)
}

pub async fn set_setting(setting: ConfigItem) -> Result<()> {
    unimplemented!()
}

pub async fn force_set_setting(setting: ConfigItem) -> Result<()> {
    unimplemented!()
}

use anyhow::Result;
use sqlx;

use crate::{consts::ConfigParams, objects::query_results::ConfigItem};

use super::manager::get_sqlite_connection;

pub async fn get_setting(setting: ConfigParams) -> Result<ConfigItem> {
    let result = sqlx::query_as::<_, ConfigItem>("SELECT * FROM config WHERE name = ?;")
        .bind(setting.to_string())
        .fetch_one(&mut get_sqlite_connection().await)
        .await?;

    Ok(result)
}

#[derive(sqlx::FromRow)]
pub struct ConfigItem {
    pub name: ConfigParams,
    pub value: String,
}

#[derive(sqlx::Type)]
pub enum ConfigParams {
    AccessCheck,
}

#[derive(sqlx::FromRow)]
pub struct ConfigItem {
    pub name: String,
    pub value: String,
}

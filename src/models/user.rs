use sqlx::types::chrono::NaiveDateTime;



#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

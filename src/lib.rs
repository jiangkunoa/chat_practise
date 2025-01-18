pub mod user;
pub mod jwt;


pub struct AppState {
    pub pool: sqlx::mysql::MySqlPool,
}
use std::env;

use sqlx::Connection;


#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let url = env::var("DATABASE_URL")?;
    let mut conn = sqlx::mysql::MySqlConnection::connect(url.as_str())
        .await?;

    sqlx::query("DROP TABLE IF EXISTS users")
        .execute(&mut conn)
        .await?;
    Ok(())
}
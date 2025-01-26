use anyhow::Result;
use sqlx::MySqlPool;

use crate::models::user::User;


pub async fn get_user(pool: &MySqlPool, id: u64) -> Option<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn get_user_by_name(pool: &MySqlPool, username: &str) -> Option<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn insert_user(pool: &MySqlPool, username: &str, password_hash: &str) -> Result<()> {
    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}

pub async fn update_user_password(pool: &MySqlPool, id: u64, password_hash: &str) -> Result<()> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}
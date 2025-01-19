use actix_web::{post, web, HttpResponse, Responder};
use log::info;
use serde::Deserialize;
use sqlx::{types::chrono::NaiveDateTime, MySqlPool};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};
use anyhow::{Context, Result};

use crate::{auth::ClaimsExtractor, jwt::build_token, ApiResponse, AppState};

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct ReqRegister {
    pub username: String,
    pub password: String,
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, user: web::Json<ReqRegister>) -> impl Responder {
    let user = user.into_inner();
    match _register(&state.pool, user).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok()),
        Err(e) => {
            log::error!("注册失败: {}", e);
            HttpResponse::Ok().json(ApiResponse::code_err(-1, format!("注册失败:{}", e.to_string())))
        }
    }
}

async fn _register(pool: &MySqlPool, user: ReqRegister) -> Result<()> {
    let old = sqlx::query_as::<_, User>("select * from users where username = ?")
        .bind(&user.username)
        .fetch_optional(pool)
        .await.context("查询失败")?;
    if let Some(_) = old {
        return Err(anyhow::anyhow!("用户名已存在"));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(user.password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))
        .context("密码哈希失败")?
        .to_string();
    info!("password: {}, password_hash: {}", user.password, password_hash);
    sqlx::query("insert into users (username, password_hash) values (?, ?)")
        .bind(&user.username)
        .bind(&password_hash)
        .execute(pool)
        .await
        .context("插入用户失败")?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ReqLogin {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(state: web::Data<AppState>, user: web::Json<ReqLogin>) -> impl Responder {
    let user = user.into_inner();
    match _login(&state.pool, user).await {
        Ok(user) =>  {
            match build_token(user.id) {
                Ok(token) => HttpResponse::Ok().json(ApiResponse::success(token)),
                Err(e) => {
                    log::error!("生成token失败: {}", e);
                    HttpResponse::Ok().json(ApiResponse::code_err(-1, format!("生成token失败：{}", e.to_string())))
                }
            }
        }
        Err(e) => {
            log::error!("登录失败: {}", e);
            HttpResponse::Ok().json(ApiResponse::code_err(-1, format!("登录失败：{}", e.to_string())))
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct ReqUpdatePassword {
    pub old_password: String,
    pub new_password: String,
}

#[post("/update_password")]
pub async fn update_password(state: web::Data<AppState>, user: web::Json<ReqUpdatePassword>, claims: ClaimsExtractor) -> impl Responder {
    let user = user.into_inner();
    let claims = claims.0;
    info!("update_password: {:?}, {:?}", user, claims);
    HttpResponse::Ok().json(ApiResponse::msg_ok("修改密码成功"));
    match _update_password(&state.pool, user, claims.sub).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::msg_ok("修改密码成功")),
        Err(e) => {
            log::error!("修改密码失败: {}", e);
            HttpResponse::Ok().json(ApiResponse::code_err(-1, format!("修改密码失败：{}", e.to_string())))
        },
    }
}

async fn _update_password(pool: &MySqlPool, req: ReqUpdatePassword, id: u64) -> Result<()> {
    let user = sqlx::query_as::<_, User>("select * from users where id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await.context("查询失败")?;

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
    let is_valid = argon2.verify_password(req.old_password.as_bytes(), &parsed_hash).is_ok();
    if !is_valid {
        return Err(anyhow::anyhow!("旧密码错误"));
    }
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(req.new_password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))
        .context("密码哈希失败")?
        .to_string();
    sqlx::query("update users set password_hash = ? where id = ?")
        .bind(&password_hash)
        .bind(user.id)
        .execute(pool)
        .await
        .context("修改密码失败")?;
    Ok(())
}

async fn _login(pool: &MySqlPool, req: ReqLogin) -> Result<User> {
    let user = sqlx::query_as::<_, User>("select * from users where username = ?")
        .bind(&req.username)
        .fetch_one(pool)
        .await.context("查询失败")?;

    let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
    let is_valid = Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_ok();

    if !is_valid {
        return Err(anyhow::anyhow!("密码错误"));
    }
    log::info!("登录成功: {}", user.username);
    Ok(user)
}


#[cfg(test)]
mod tests {
    use argon2::{PasswordHash, PasswordVerifier};

    use super::*;

    #[test]
    fn test_hash() {
        println!("test begin");
        let password = "123456";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!(e))
            .context("密码哈希失败").unwrap()
            .to_string();
        println!("password_hash: {}", password_hash);
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        let is_valid = argon2.verify_password("123456".as_bytes(), &parsed_hash).is_ok();
        assert!(is_valid);
        println!("is_valid: {}", is_valid);
    }
}
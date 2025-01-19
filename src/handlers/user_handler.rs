use actix_web::{post, web, HttpResponse, Responder};
use log::info;
use serde::Deserialize;
use sqlx::MySqlPool;
use anyhow::Result;

use crate::{dao::user_dao::*, models::user::User, utils::argon2::{password_hash, password_verify}, web::{auth::ClaimsExtractor, common::{ApiResponse, AppState}, jwt::build_token}};


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
    let old = get_user_by_name(pool, &user.username).await;
    if let Some(_) = old {
        return Err(anyhow::anyhow!("用户名已存在"));
    }
    let password_hash = password_hash(&user.password)?;
    info!("password: {}, password_hash: {}", user.password, password_hash);
    insert_user(pool, &user.username, password_hash.as_str()).await?;
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
    let user = get_user(pool, id).await.ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
    if !password_verify(&req.old_password, &user.password_hash)? {
        return Err(anyhow::anyhow!("旧密码错误"));
    }
    let password_hash = password_hash(&req.new_password)?;
    update_user_password(pool, id, password_hash.as_str()).await?;
    Ok(())
}

async fn _login(pool: &MySqlPool, req: ReqLogin) -> Result<User> {
    let user = get_user_by_name(pool, &req.username).await.ok_or_else(|| anyhow::anyhow!("用户不存在"))?;
    if !password_verify(&req.password, &user.password_hash)? {
        return Err(anyhow::anyhow!("密码错误"));
    }
    log::info!("登录成功: {}", user.username);
    Ok(user)
}


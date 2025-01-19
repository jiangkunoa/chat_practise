use actix_web::web;

pub mod user_router;

pub fn config_router(cfg: &mut web::ServiceConfig) {
    // 注册用户路由
    user_router::config(cfg);
}
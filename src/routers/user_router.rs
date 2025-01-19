use actix_web::web;

use crate::handlers::user_handler;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_handler::register)
    .service(user_handler::login)
    .service(user_handler::update_password);
}
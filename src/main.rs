use std::env;

use actix_web::{get, web::Data, App, HttpServer};
use chat_practise::{chat::chatserver::start_chat_server, routers::config_router, web::{auth::AuthMiddleware, common::AppState}};
use sqlx::mysql::MySqlPoolOptions;

#[get("/")]
async fn greet() -> String {
    format!("Hello !")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let url = env::var("DATABASE_URL").expect("ENV DATABASE_URL ERROR");
    let chat_port: u16 = env::var("CHAT_PORT").unwrap_or("8081".to_string()).parse().expect("ENV CHAT_PORT ERROR");
    let web_port: u16 = env::var("WEB_PORT").unwrap_or("8080".to_string()).parse().expect("ENV WEB_PORT ERROR");
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("trace"));

    let pool = MySqlPoolOptions::new()
    .max_connections(5)
    .connect(url.as_str())
    .await
    .unwrap();
    let chat_state = start_chat_server(chat_port, pool.clone()).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    HttpServer::new(move || {
        App::new()
        .app_data(Data::new(AppState { pool: pool.clone() }))
        .app_data(Data::new(chat_state.clone()))
        .wrap(AuthMiddleware {
            whitelist:vec!["/login".to_owned(), "/register".to_owned()]
        })
        .configure(config_router)
    })
    .bind(("127.0.0.1", web_port))?
    .run()
    .await
}


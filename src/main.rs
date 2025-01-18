use actix_web::{get, web::Data, App, HttpServer};
use chat_practise::{user::{login, register}, AppState};
use sqlx::mysql::MySqlPoolOptions;

#[get("/")]
async fn greet() -> String {
    format!("Hello !")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("trace"));

    let pool = MySqlPoolOptions::new()
    .max_connections(5)
    .connect("mysql://root:jiangkunoa@192.168.1.17:3306/chat")
    .await
    .unwrap();
    println!("{:?}",pool);

    HttpServer::new(move || {
        App::new()
        .app_data(Data::new(AppState { pool: pool.clone() }))
        .service(register)
        .service(login)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


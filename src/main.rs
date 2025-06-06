use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use env_logger::Builder;
use log::LevelFilter;
use actix_web::cookie::Key;

mod db;
mod news;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    Builder::new()
       .filter(None, LevelFilter::Info)
       .init();

    // 加载环境变量
    dotenv().ok();

    // 初始化数据库连接池
    let database_url = env::var("DATABASE_URL")
       .expect("DATABASE_URL must be set (e.g., postgres://postgres:wcyyds551...@localhost/news)");

    let pool = PgPoolOptions::new()
       .max_connections(10)
       .connect(&database_url)
       .await
       .expect("Failed to create DB pool");

    // 初始化服务
    let news_repo = db::repo::NewsRepo::new(pool.clone());
    let news_dao = news::dao::NewsDao::new(news_repo);
    let users_repo = db::repo::UsersRepo::new(pool.clone());
    let news_service = news::service::NewsService::new(news_dao, users_repo);

    // 生成会话密钥
    let secret_key = Key::generate();

    // 启动HTTP服务器
    HttpServer::new(move || {
        App::new()
           .wrap(middleware::Logger::default())
           .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone()
            ))
           .app_data(web::Data::new(news_service.clone()))
           .configure(news::routes::config)
           .service(web::resource("/").to(|| async {
                HttpResponse::Ok().content_type("text/html").body(include_str!("../src/static/index.html"))
            }))
           .service(actix_files::Files::new("/", "./src/static").show_files_listing())
    })
   .bind(("0.0.0.0", 8080))?
   .run()
   .await
}
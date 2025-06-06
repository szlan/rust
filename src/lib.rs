pub mod db;
pub mod news;

// 公共类型定义
pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type AppDbPool = sqlx::Pool<sqlx::Postgres>;

// 导出核心模块
pub use db::repo::{NewsRepo, UsersRepo};
pub use news::{
    dao::NewsDao, 
    service::NewsService, 
    models::{NewsCreate, NewsQuery}
};
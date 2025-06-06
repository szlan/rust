use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

pub async fn init_pool() -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g., postgres://postgres:password@localhost/news)");

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
}



// 在 db/pool.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[actix_rt::test]
    async fn test_init_pool() {
        dotenv().ok();
        let pool = init_pool().await;
        assert!(pool.is_ok(), "Failed to initialize database pool");
    }
}
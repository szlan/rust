use crate::db::models::{NewsModel, UserModel};
use sqlx::{PgPool, Error};
use chrono::{Utc, NaiveDateTime};

// 新闻仓库实现
#[derive(Clone)]  // 新增 Clone 派生
pub struct NewsRepo {
    pool: PgPool,
}

impl NewsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 创建新闻（强制非空字段）
    pub async fn create_news(
        &self,
        news_type: &str,
        href: &str,
        title: &str,
        content: &str,
    ) -> Result<NewsModel, Error> {
        let created_at = Utc::now().naive_utc();
        sqlx::query_as!(
            NewsModel,
            r#"
            INSERT INTO news (
                news_type, href, title, datetime, content
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id, news_type, href, title, datetime, content
            "#,
            news_type,
            href,
            title,
            created_at,
            content
        )
        .fetch_one(&self.pool)
        .await
    }


    // 修改 get_paginated 方法的参数和 offset 计算
    pub async fn get_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<NewsModel>, Error> {
        // 安全计算 offset（避免 u32 溢出，若 page 或 page_size 可能过大，建议用 i64 参数）
        let offset = (page.saturating_sub(1) as i64) * (page_size as i64); // 转为 i64 避免溢出
        sqlx::query_as!(
            NewsModel,
            r#"
            SELECT
                id,
                news_type,
                href,
                title,
                datetime,
                content
            FROM news
            ORDER BY datetime DESC
            LIMIT $1 OFFSET $2
            "#,
            page_size as i64, // LIMIT 接受 i64
            offset // 已为 i64
        )
        .fetch_all(&self.pool)
        .await
    }
    
    /* 
    // 在 NewsRepo 实现中添加
    pub async fn get_news_count(&self, category: Option<&str>) -> Result<i64, Error> {
        match category {
            Some(cat) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM news WHERE news_type = $1",
                    cat
                )
                .fetch_one(&self.pool)
                .await
            }
            None => {
                sqlx::query_scalar!("SELECT COUNT(*) FROM news")
                    .fetch_one(&self.pool)
                    .await
            }
        }
    }
    */
    ///*
    // 修改 get_news_count 方法，处理 Option<i64>
pub async fn get_news_count(&self, category: Option<&str>) -> Result<i64, Error> {
    let count = match category {
        Some(cat) => {
            // 执行带条件的查询，处理 None 为 0
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM news WHERE news_type = $1",
                cat
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0) // 无结果时返回 0
        }
        None => {
            // 执行全表查询，处理 None 为 0
            sqlx::query_scalar!("SELECT COUNT(*) FROM news")
                .fetch_one(&self.pool)
                .await?
                .unwrap_or(0) // 无结果时返回 0
        }
    };
    Ok(count)
}
    //*/

    // 在 NewsRepo 实现中添加
    pub async fn get_paginated_by_category(
        &self,
        limit: u32,
        offset: i64,
        category: &str,
    ) -> Result<Vec<NewsModel>, Error> {
        sqlx::query_as!(
            NewsModel,
            r#"
            SELECT id, news_type, href, title, datetime, content
            FROM news
            WHERE news_type = $1
            ORDER BY datetime DESC
            LIMIT $2 OFFSET $3
            "#,
            category,
            limit as i64,
            offset
        )
        .fetch_all(&self.pool)
        .await
    }


}


// 用户仓库实现
#[derive(Clone)]  // 新增 Clone 派生（如果 UsersRepo 也需要克隆）
pub struct UsersRepo {
    pool: PgPool,
}

impl UsersRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // 创建用户（强制非空字段）
    pub async fn create_user(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<UserModel, Error> {
        sqlx::query_as!(
            UserModel,
            r#"
            INSERT INTO users (name, email, password)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, password
            "#,
            name,
            email,
            password
        )
        .fetch_one(&self.pool)
        .await
    }

    // 根据邮箱查询用户
    pub async fn get_user_by_email(&self, email: &str) -> Result<UserModel, Error> {
        sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, name, email, password
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
    }

    // 根据用户ID查询用户信息
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<UserModel, Error> {
        sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, name, email, password
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
    }
}

// 在 db/repo.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_pool;
    use sqlx::PgPool;

    #[actix_rt::test]
    async fn test_create_and_query_news() {
        let pool = init_pool().await.unwrap();
        let repo = NewsRepo::new(pool.clone());

        // 使用事务回滚，避免污染测试数据库
        let mut tx = pool.begin().await.unwrap();

        // 测试创建新闻
        let news = repo
           .create_news("tech", "https://test.com", "Test News", "Content")
           .await
           .unwrap();
        assert_eq!(news.title, "Test News");

        // 测试分页查询
        let result = repo.get_paginated(1, 10).await.unwrap();
        assert!(!result.is_empty());

        tx.rollback().await.unwrap(); // 回滚事务
    }

    #[actix_rt::test]
    async fn test_create_and_query_user() {
        let pool = init_pool().await.unwrap();
        let repo = UsersRepo::new(pool.clone());

        // 使用事务回滚，避免污染测试数据库
        let mut tx = pool.begin().await.unwrap();

        // 生成唯一的邮箱地址，避免冲突
        let timestamp = Utc::now().timestamp_nanos();
        let unique_email = format!("test_{}@example.com", timestamp);

        // 测试创建用户
        let user = repo
        .create_user("test_user", &unique_email, "test_password")
        .await
        .unwrap();
        assert_eq!(user.name, "test_user");
        assert_eq!(user.email, unique_email);

        // 测试根据邮箱查询用户
        let queried_user = repo.get_user_by_email(&unique_email).await.unwrap();
        assert_eq!(queried_user.id, user.id);
        assert_eq!(queried_user.name, user.name);
        assert_eq!(queried_user.email, user.email);
        assert_eq!(queried_user.password, user.password);

        tx.rollback().await.unwrap(); // 回滚事务
    }
}
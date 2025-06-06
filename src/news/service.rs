use crate::db::repo::{NewsRepo, UsersRepo};
use crate::db::models::{NewsModel, UserModel, UserRegister, UserLogin};
use crate::news::dao::NewsDao;
use crate::news::models::{NewsCreate, NewsQuery};
use crate::news::models::PaginatedNews;
use thiserror::Error;
use sqlx::Error as SqlxError;
use chrono::Utc;

#[derive(Debug, Error)]
pub enum NewsError {
    #[error("Database error: {0}")]
    DbError(#[from] SqlxError),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid password")]
    InvalidPassword,
}

#[derive(Clone)]  // 新增 Clone 派生
pub struct NewsService {
    news_dao: NewsDao,
    users_repo: UsersRepo,
}

impl NewsService {
    pub fn new(news_dao: NewsDao, users_repo: UsersRepo) -> Self {
        Self { news_dao, users_repo }
    }

    // 创建新闻（含基础验证）
    pub async fn create_news(
        &self,
        data: NewsCreate,
    ) -> Result<NewsModel, NewsError> {
        // 简单字段校验
        if data.title.is_empty() {
            return Err(NewsError::ValidationError("Title cannot be empty".into()));
        }
        if data.content.is_empty() {
            return Err(NewsError::ValidationError("Content cannot be empty".into()));
        }

        self.news_dao
           .create_news(&data.news_type, &data.href, &data.title, &data.content)
           .await
           .map_err(NewsError::DbError)
    }


    /*   // 取消分页查询新闻，因为它会被替换为新的分页逻辑

    // 分页查询新闻
    pub async fn get_paginated(
        &self,
        query: NewsQuery,
    ) -> Result<Vec<NewsModel>, NewsError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        self.news_dao
           .get_paginated(page, page_size)
           .await
           .map_err(NewsError::DbError)
    }
    */


    // 修改 get_paginated 方法
    pub async fn get_paginated(
        &self,
        query: NewsQuery,
    ) -> Result<PaginatedNews, NewsError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10);
        let category = query.category.as_deref();
        
        self.news_dao
            .get_paginated_with_count(page, page_size, category)
            .await
            .map_err(NewsError::DbError)
    }


    // 用户注册
    pub async fn register_user(
        &self,
        data: UserRegister,
    ) -> Result<UserModel, NewsError> {
        // 简单字段校验
        if data.name.is_empty() {
            return Err(NewsError::ValidationError("Name cannot be empty".into()));
        }
        if data.email.is_empty() {
            return Err(NewsError::ValidationError("Email cannot be empty".into()));
        }
        if data.password.is_empty() {
            return Err(NewsError::ValidationError("Password cannot be empty".into()));
        }

        // 检查用户是否已存在
        match self.users_repo.get_user_by_email(&data.email).await {
            Ok(_) => return Err(NewsError::ValidationError("User already exists".into())),
            Err(_) => {}
        }

        self.users_repo
           .create_user(&data.name, &data.email, &data.password)
           .await
           .map_err(NewsError::DbError)
    }

    // 用户登录
    pub async fn login_user(
        &self,
        data: UserLogin,
    ) -> Result<UserModel, NewsError> {
        let user = self.users_repo.get_user_by_email(&data.email).await.map_err(|_| NewsError::UserNotFound)?;
        if user.password != data.password {
            return Err(NewsError::InvalidPassword);
        }
        Ok(user)
    }

    // 新增公共方法来访问 users_repo
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<UserModel, NewsError> {
        self.users_repo.get_user_by_id(user_id).await.map_err(NewsError::DbError)
    }
}

// 在 news/service.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_pool;
    use crate::news::dao::NewsDao;

    #[actix_rt::test]
    async fn test_pagination_logic() {
        let pool = init_pool().await.unwrap();
        let news_repo = NewsRepo::new(pool.clone());
        let news_dao = NewsDao::new(news_repo);
        let users_repo = UsersRepo::new(pool.clone());
        let service = NewsService::new(news_dao, users_repo);
    }

    #[actix_rt::test]
    async fn test_user_register_and_login() {
        let pool = init_pool().await.unwrap();
        let news_repo = NewsRepo::new(pool.clone());
        let news_dao = NewsDao::new(news_repo);
        let users_repo = UsersRepo::new(pool.clone());
        let service = NewsService::new(news_dao, users_repo);

        // 生成唯一的邮箱地址，避免冲突
        let timestamp = Utc::now().timestamp_nanos();
        let unique_email = format!("test_{}@example.com", timestamp);

        let register_data = UserRegister {
            name: "test_user".to_string(),
            email: unique_email.clone(), // 使用唯一邮箱
            password: "test_password".to_string(),
        };

        // 测试用户注册
        let registered_user = service.register_user(register_data.clone()).await.unwrap();
        assert_eq!(registered_user.email, unique_email);

        let login_data = UserLogin {
            email: unique_email.clone(), // 使用相同的唯一邮箱
            password: "test_password".to_string(),
        };

        // 测试用户登录
        let logged_in_user = service.login_user(login_data).await.unwrap();
        assert_eq!(logged_in_user.email, unique_email);
    }
}
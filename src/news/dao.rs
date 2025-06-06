use crate::db::repo::NewsRepo;
use crate::db::models::NewsModel;
use crate::news::models::PaginatedNews;


#[derive(Clone)]  // 新增 Clone 派生
pub struct NewsDao {
    repo: NewsRepo,
}

impl NewsDao {
    pub fn new(repo: NewsRepo) -> Self {
        Self { repo }
    }

    pub async fn create_news(
        &self,
        news_type: &str,
        href: &str,
        title: &str,
        content: &str,
    ) -> Result<NewsModel, sqlx::Error> {
        self.repo
            .create_news(news_type, href, title, content)
            .await
    }


    /* 
    pub async fn get_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<NewsModel>, sqlx::Error> {
        self.repo
            .get_paginated(page, page_size)
            .await
    }
    */

    // 在 NewsDao 实现中添加
    /* 
    pub async fn get_paginated_with_count(
        &self,
        page: u32,
        page_size: u32,
        category: Option<&str>,
        ) -> Result<PaginatedNews, sqlx::Error> {
        //let offset = (page - 1) * page_size;
        let offset = (page.saturating_sub(1) as i64) * (page_size as i64); // 转为 i64 并安全计算
        let news = if let Some(cat) = category {
            self.repo
                .get_paginated_by_category(page_size, offset as i64, cat)
                .await?
        } else {
            self.repo
                //.get_paginated(page_size, offset as i64)
                .get_paginated(page, page_size).await? // page 是 u32，无需转换为 i64
                .await?
        };

        let total_news = self.repo.get_news_count(category).await?;
        let total_pages = (total_news as f64 / page_size as f64).ceil() as i64;

        Ok(PaginatedNews {
            news,
            total_pages,
            current_page: page,
        })
    }
    */

    pub async fn get_paginated_with_count(
        &self,
        page: u32,
        page_size: u32,
        category: Option<&str>,
    ) -> Result<PaginatedNews, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let news = match category {
            Some(cat) => self.repo.get_paginated_by_category(page_size, offset as i64, cat).await?,
            None => self.repo.get_paginated(page, page_size).await?,
        };

        let total = self.repo.get_news_count(category).await?;
        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(PaginatedNews {
            news,
            total_pages,
            current_page: page,
        })
    }
}




// 在 news/dao.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_pool;

    #[actix_rt::test]
    async fn test_dao_create_news() {
        let pool = init_pool().await.unwrap();
        let repo = NewsRepo::new(pool.clone());
        let dao = NewsDao::new(repo);

        let mut tx = pool.begin().await.unwrap();

        // 测试 DAO 方法
        let result = dao
            .create_news("tech", "https://dao.test", "DAO Test", "Content")
            .await;
        assert!(result.is_ok());

        tx.rollback().await.unwrap();
    }
}
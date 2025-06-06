
use serde::{Deserialize};
use serde::Serialize;
use crate::db::models::NewsModel;

// 创建新闻的请求体
#[derive(Debug, Deserialize)]
pub struct NewsCreate {
    pub news_type: String,
    pub href: String,
    pub title: String,
    pub content: String,
}

// 查询新闻的请求参数
#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub category: Option<String>, // 添加这个字段
}


// 在 news/models.rs 中添加
#[derive(Debug, Serialize)]
pub struct PaginatedNews {
    pub news: Vec<NewsModel>,
    pub total_pages: i64,
    pub current_page: u32,
}


// 在 news/models.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_create_deserialization() {
        let json = r#"{
            "news_type": "tech",
            "href": "https://test.com",
            "title": "Test",
            "content": "Content"
        }"#;

        let data: NewsCreate = serde_json::from_str(json).unwrap();
        assert_eq!(data.news_type, "tech");
    }
}
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

// 新闻模型（严格匹配数据库表结构）
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NewsModel {
    pub id: i32,
    pub news_type: String,        // 数据库字段为 NOT NULL
    pub href: String,             // 数据库字段为 NOT NULL
    pub title: String,            // 数据库字段为 NOT NULL
    pub datetime: NaiveDateTime,  // 数据库字段为 NOT NULL
    pub content: String,          // 数据库字段为 NOT NULL
}

// 用户模型（严格匹配数据库表结构）
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserModel {
    pub id: i32,
    pub name: String,             // 数据库字段为 NOT NULL
    pub email: String,            // 数据库字段为 NOT NULL
    pub password: String,         // 数据库字段为 NOT NULL
}

// 用户注册请求体
#[derive(Debug, Deserialize,Clone)]
pub struct UserRegister {
    pub name: String,
    pub email: String,
    pub password: String,
}

// 用户登录请求体
#[derive(Debug, Deserialize,Clone)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

// 在 db/models.rs 底部添加以下测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_news_model_serialization() {
        let datetime = Local::now().naive_utc();
        let news = NewsModel {
            id: 1,
            news_type: "tech".into(),
            href: "https://example.com".into(),
            title: "Test".into(),
            datetime,
            content: "Content".into(),
        };

        // 测试序列化
        let serialized = serde_json::to_string(&news).unwrap();
        assert!(serialized.contains("\"news_type\":\"tech\""));

        // 测试反序列化
        let deserialized: NewsModel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.title, "Test");
    }
}
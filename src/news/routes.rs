// src/routes.rs

use actix_web::{http, web, HttpResponse, Responder, HttpRequest, post}; // 确保引入 post 宏
use actix_web::App; // App 可能在 main.rs 中使用
use actix_session::Session;
use crate::{
    // db::repo::{NewsRepo, UsersRepo}, // 这些在 service 层使用，handler 层不直接用 repo
    db::models::{UserRegister, UserLogin}, // 如果 handler 需要直接处理这些模型
    news::{
        // dao::NewsDao, // 同上，handler 通过 service 交互
        service::NewsService,
        models::{NewsCreate, NewsQuery},
    },
};
use serde_json; // 确保引入

// 新闻路由配置
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/news")
           .route("", web::post().to(create_news))
           .route("", web::get().to(list_news))
    );
    cfg.service(
        web::scope("/user")
           .route("/register", web::post().to(register_user))
           .route("/login", web::post().to(login_user))
           .route("/check-login", web::get().to(check_user_login))
           .route("/logout", web::post().to(logout_user)) // <--- 新增登出路由
    );
}

// ... (create_news, list_news, register_user, login_user, check_user_login 函数保持不变) ...


// ++++++++++++++++++++++++++++++++++++++++++++++++++
// 新增：用户登出接口
// ++++++++++++++++++++++++++++++++++++++++++++++++++
async fn logout_user(session: Session) -> impl Responder {
    log::info!("logout_user called");

    // 清除 session 中的所有数据
    session.purge(); // actix-session 提供的便捷方法来销毁会话数据

    // 或者，如果你只想移除特定的键：
    // session.remove("user_id");

    // 返回成功响应
    HttpResponse::Ok().json(serde_json::json!({
        "message": "成功退出登录"
    }))
}
// ++++++++++++++++++++++++++++++++++++++++++++++++++


// 原有的 create_news, list_news, register_user, login_user, check_user_login 函数：

// 创建新闻接口
async fn create_news(
    service: web::Data<NewsService>,
    req: web::Json<NewsCreate>,
) -> impl Responder {
    log::info!("create_news called");
    match service.create_news(req.into_inner()).await {
        Ok(news) => HttpResponse::Created().json(news),
        Err(e) => {
            log::error!("Error creating news: {}", e); // 添加日志
            HttpResponse::InternalServerError().json(serde_json::json!({ // 或者更具体的错误码
                "message": format!("Error: {}", e)
            }))
        }
    }
}


/*
// 分页查询新闻接口
async fn list_news(
    service: web::Data<NewsService>,
    query: web::Query<NewsQuery>,
) -> impl Responder {
    log::info!("list_news called");
    match service.get_paginated(query.into_inner()).await {
        Ok(news_list) => HttpResponse::Ok().json(news_list),
        Err(e) => {
            log::error!("Error listing news: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Error: {}", e)
            }))
        }
    }
}
*/

// 修改 list_news 函数
async fn list_news(
    service: web::Data<NewsService>,
    query: web::Query<NewsQuery>,
) -> impl Responder {
    log::info!("list_news called with query: {:?}", query);
    
    match service.get_paginated(query.into_inner()).await {
        Ok(paginated_news) => HttpResponse::Ok().json(paginated_news),
        Err(e) => {
            log::error!("Error listing news: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Error: {}", e)
            }))
        }
    }
}


// 用户注册接口
async fn register_user(
    service: web::Data<NewsService>,
    req: web::Json<UserRegister>,
) -> impl Responder {
    log::info!("register_user called");
    match service.register_user(req.into_inner()).await {
        Ok(user) => HttpResponse::Created().json(serde_json::json!({
            "message": "User registered successfully",
            "user": user // 你可能只想返回部分用户信息，而不是整个模型（包含密码哈希等）
        })),
        Err(e) => { // 根据错误类型返回不同状态码
            log::warn!("Registration failed: {}", e);
            // 假设 NewsError::ValidationError 来自 service 层
            // if let NewsError::ValidationError(msg) = e { // 如果你的 NewsError 定义了 ValidationError
            //     return HttpResponse::BadRequest().json(serde_json::json!({"message": msg}));
            // }
            HttpResponse::BadRequest().json(serde_json::json!({
                "message": format!("Registration failed: {}", e) // 这里的e是NewsError
            }))
        }
    }
}

// 用户登录接口
async fn login_user(
    service: web::Data<NewsService>,
    req: web::Json<UserLogin>,
    session: Session,
) -> impl Responder {
    log::info!("login_user called");
    match service.login_user(req.into_inner()).await {
        Ok(user) => {
            // 在session中存储用户ID或其他必要信息
            // 注意：unwrap() 在生产代码中应谨慎使用，最好处理Error
            if let Err(e) = session.insert("user_id", user.id) {
                log::error!("Failed to insert user_id into session: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "message": "Session error during login"
                }));
            }
            // 你可能还想存储用户名等，方便check-login时直接使用或显示
            // if let Err(e) = session.insert("username", user.name.clone()) { ... }

            log::info!("User {} logged in successfully, session id stored.", user.id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "User logged in successfully",
                // 出于安全考虑，通常不应在登录响应中返回完整的用户信息（尤其是密码）
                // 可以只返回部分信息或一个token（如果使用JWT）
                "user": { "id": user.id, "name": user.name, "email": user.email }
            }))
        }
        Err(e) => {
            log::warn!("Login failed: {}", e);
            HttpResponse::Unauthorized().json(serde_json::json!({
                "message": format!("Login failed: {}", e)
            }))
        }
    }
}

// 检查用户登录状态接口
async fn check_user_login(
    service: web::Data<NewsService>, // 仍然需要 service 来获取用户信息
    session: Session,
    _req: HttpRequest, // HttpRequest 可能不需要，除非你要检查请求头等
) -> impl Responder {
    log::info!("check_user_login called");
    // 尝试从 session 获取 user_id
    match session.get::<i32>("user_id") {
        Ok(Some(user_id)) => {
            log::info!("User ID {} found in session.", user_id);
            // 根据 user_id 从数据库获取用户信息
            // (你的 NewsService 已经有 get_user_by_id 方法)
            match service.get_user_by_id(user_id).await {
                Ok(user_model) => {
                    // 出于安全，不返回密码等敏感信息
                    HttpResponse::Ok().json(serde_json::json!({
                        "id": user_model.id,
                        "name": user_model.name,
                        "email": user_model.email
                        // 不应包含 password
                    }))
                }
                Err(e) => {
                    log::error!("Failed to get user by ID {} from service: {}", user_id, e);
                    // 用户ID在session中，但数据库中找不到，可能是数据不一致或用户已被删除
                    session.purge(); // 清除无效的session
                    HttpResponse::Unauthorized().json(serde_json::json!({
                        "message": "User not found or session invalid"
                    }))
                }
            }
        }
        Ok(None) => {
            log::info!("No user_id found in session.");
            HttpResponse::Unauthorized().json(serde_json::json!({
                "message": "User not logged in" // 更明确的未登录消息
            }))
        }
        Err(e) => {
            log::error!("Error getting user_id from session: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Session error"
            }))
        }
    }
}
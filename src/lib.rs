pub mod api;
pub mod config;
pub mod error;
pub mod middleware;
pub mod model;
pub mod repository;
pub mod service;
pub mod utils;

pub use api::{auth, user};
pub use config::AppConfig;
pub use error::{AppError, AppResult};
pub use service::{AuthService, WechatService};

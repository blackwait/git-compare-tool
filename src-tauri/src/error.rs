use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("配置文件读写失败: {0}")]
    ConfigIo(String),
    #[error("路径不是 Git 仓库: {0}")]
    NotARepo(String),
    #[error("工作区不存在: {0}")]
    WorkspaceNotFound(String),
    #[error("未找到 git 可执行文件")]
    GitNotFound,
    #[error("Git 命令失败: {0}")]
    GitFailed(String),
    #[error("命令超时")]
    Timeout,
    #[error("参数错误: {0}")]
    InvalidArg(String),
    #[error("IO: {0}")]
    Io(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ConfigIo(e.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

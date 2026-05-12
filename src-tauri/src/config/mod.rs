pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub default_view: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            default_view: "tree".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub workspaces: Vec<Workspace>,
    pub settings: Settings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            workspaces: vec![],
            settings: Settings::default(),
        }
    }
}

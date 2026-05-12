use super::AppConfig;
use crate::error::{AppError, AppResult};
use std::{fs, path::PathBuf};

pub struct ConfigStore {
    pub path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(AppError::from)?;
        }
        Ok(Self { path })
    }

    pub fn load(&self) -> AppResult<AppConfig> {
        if !self.path.exists() {
            return Ok(AppConfig::default());
        }
        let raw = fs::read_to_string(&self.path).map_err(AppError::from)?;
        match serde_json::from_str::<AppConfig>(&raw) {
            Ok(c) => Ok(c),
            Err(e) => {
                let bak = self.path.with_extension("json.bak");
                let _ = fs::copy(&self.path, &bak);
                let fresh = AppConfig::default();
                self.save(&fresh)?;
                tracing::warn!("config corrupted ({}); reset to default, backup at {:?}", e, bak);
                Ok(fresh)
            }
        }
    }

    pub fn save(&self, cfg: &AppConfig) -> AppResult<()> {
        let tmp = self.path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(cfg)?;
        fs::write(&tmp, json).map_err(AppError::from)?;
        fs::rename(&tmp, &self.path).map_err(AppError::from)?;
        Ok(())
    }

    pub fn mutate<F>(&self, f: F) -> AppResult<AppConfig>
    where
        F: FnOnce(&mut AppConfig) -> AppResult<()>,
    {
        let mut cfg = self.load()?;
        f(&mut cfg)?;
        self.save(&cfg)?;
        Ok(cfg)
    }
}

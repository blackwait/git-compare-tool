pub mod git;
pub mod workspace;

use crate::config::store::ConfigStore;
use std::sync::Mutex;

pub struct AppState {
    pub store: Mutex<ConfigStore>,
}

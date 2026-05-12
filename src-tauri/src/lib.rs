mod commands;
mod config;
mod error;
mod git;

use commands::AppState;
use config::store::ConfigStore;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base = app
                .path()
                .app_data_dir()
                .expect("appDataDir");
            let dir = base.join("git-compare-tool");
            std::fs::create_dir_all(&dir).ok();
            let path = dir.join("config.json");
            let store = ConfigStore::new(path).expect("init config store");
            app.manage(AppState {
                store: Mutex::new(store),
            });

            // 启动时探测 git 是否可用
            tauri::async_runtime::spawn(async {
                match crate::git::runner::run(".", &["--version"], 5).await {
                    Ok(_) => tracing::info!("git is available"),
                    Err(e) => tracing::warn!("git 不可用: {e}"),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::config_load,
            commands::workspace::workspace_list,
            commands::workspace::workspace_create,
            commands::workspace::workspace_update,
            commands::workspace::workspace_delete,
            commands::workspace::workspace_pick_dir,
            commands::workspace::settings_save,
            commands::git::git_validate_repo,
            commands::git::git_list_branches,
            commands::git::git_current_branch,
            commands::git::git_fetch,
            commands::git::git_fetch_and_pull,
            commands::git::git_pull_branch,
            commands::git::git_diff_branches,
            commands::git::git_file_diff,
            commands::git::git_file_content,
            commands::git::git_log_between,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

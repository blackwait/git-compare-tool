use crate::commands::AppState;
use crate::config::{AppConfig, Workspace};
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn normalize(p: &str) -> String {
    p.replace('\\', "/")
}

#[tauri::command]
pub fn config_load(state: State<'_, AppState>) -> AppResult<AppConfig> {
    state.store.lock().unwrap().load()
}

#[tauri::command]
pub fn workspace_list(state: State<'_, AppState>) -> AppResult<Vec<Workspace>> {
    Ok(state.store.lock().unwrap().load()?.workspaces)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArgs {
    pub name: String,
    pub path: String,
    pub remote_name: Option<String>,
}

#[tauri::command]
pub fn workspace_create(state: State<'_, AppState>, args: CreateArgs) -> AppResult<Workspace> {
    if args.name.trim().is_empty() {
        return Err(AppError::InvalidArg("name 不能为空".into()));
    }
    if args.path.trim().is_empty() {
        return Err(AppError::InvalidArg("path 不能为空".into()));
    }
    let ws = Workspace {
        id: format!("ws_{}", ulid::Ulid::new()),
        name: args.name,
        path: normalize(&args.path),
        remote_name: args.remote_name.or_else(|| Some("origin".into())),
        created_at: now_ms(),
        updated_at: now_ms(),
    };
    let id = ws.id.clone();
    let ws_clone = ws.clone();
    let cfg = state
        .store
        .lock()
        .unwrap()
        .mutate(|c| {
            c.workspaces.push(ws_clone);
            Ok(())
        })?;
    Ok(cfg.workspaces.into_iter().find(|w| w.id == id).unwrap())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArgs {
    pub id: String,
    pub patch: PatchWorkspace,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchWorkspace {
    pub name: Option<String>,
    pub path: Option<String>,
    pub remote_name: Option<String>,
}

#[tauri::command]
pub fn workspace_update(state: State<'_, AppState>, args: UpdateArgs) -> AppResult<Workspace> {
    let id = args.id.clone();
    let cfg = state.store.lock().unwrap().mutate(|c| {
        let w = c
            .workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or_else(|| AppError::WorkspaceNotFound(id.clone()))?;
        if let Some(v) = args.patch.name {
            w.name = v;
        }
        if let Some(v) = args.patch.path {
            w.path = normalize(&v);
        }
        if let Some(v) = args.patch.remote_name {
            w.remote_name = Some(v);
        }
        w.updated_at = now_ms();
        Ok(())
    })?;
    Ok(cfg.workspaces.into_iter().find(|w| w.id == args.id).unwrap())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteArgs {
    pub id: String,
}

#[tauri::command]
pub fn workspace_delete(state: State<'_, AppState>, args: DeleteArgs) -> AppResult<()> {
    state.store.lock().unwrap().mutate(|c| {
        let before = c.workspaces.len();
        c.workspaces.retain(|w| w.id != args.id);
        if c.workspaces.len() == before {
            return Err(AppError::WorkspaceNotFound(args.id.clone()));
        }
        Ok(())
    })?;
    Ok(())
}

#[tauri::command]
pub async fn workspace_pick_dir(app: tauri::AppHandle) -> AppResult<Option<String>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |p| {
        let res = p.map(|path| path.to_string());
        let _ = tx.send(res);
    });
    Ok(rx.await.unwrap_or(None).map(|s| normalize(&s)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsArgs {
    pub theme: String,
    pub default_view: String,
}

#[tauri::command]
pub fn settings_save(state: State<'_, AppState>, args: SettingsArgs) -> AppResult<()> {
    state.store.lock().unwrap().mutate(|c| {
        c.settings.theme = args.theme;
        c.settings.default_view = args.default_view;
        Ok(())
    })?;
    Ok(())
}

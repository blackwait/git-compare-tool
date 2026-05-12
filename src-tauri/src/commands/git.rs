use crate::commands::AppState;
use crate::config::Workspace;
use crate::error::{AppError, AppResult};
use crate::git::{branches, diff, log as git_log};
use serde::{Deserialize, Serialize};
use tauri::State;

fn find_ws(state: &State<'_, AppState>, id: &str) -> AppResult<Workspace> {
    state
        .store
        .lock()
        .unwrap()
        .load()?
        .workspaces
        .into_iter()
        .find(|w| w.id == id)
        .ok_or_else(|| AppError::WorkspaceNotFound(id.into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateArgs {
    pub path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResult {
    pub valid: bool,
    pub root: Option<String>,
}

#[tauri::command]
pub async fn git_validate_repo(args: ValidateArgs) -> AppResult<ValidateResult> {
    let (valid, root) = branches::validate_repo(&args.path).await?;
    Ok(ValidateResult { valid, root })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoIdArgs {
    pub repo_id: String,
}

#[tauri::command]
pub async fn git_list_branches(
    state: State<'_, AppState>,
    args: RepoIdArgs,
) -> AppResult<Vec<branches::Branch>> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::list(&ws.path).await
}

#[tauri::command]
pub async fn git_current_branch(
    state: State<'_, AppState>,
    args: RepoIdArgs,
) -> AppResult<String> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::current(&ws.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchArgs {
    pub repo_id: String,
    pub remote: Option<String>,
}

#[tauri::command]
pub async fn git_fetch(state: State<'_, AppState>, args: FetchArgs) -> AppResult<()> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::fetch(&ws.path, args.remote.as_deref()).await
}

#[tauri::command]
pub async fn git_fetch_and_pull(state: State<'_, AppState>, args: RepoIdArgs) -> AppResult<()> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::fetch_and_pull(&ws.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullBranchArgs {
    pub repo_id: String,
    pub branch: String,
}

#[tauri::command]
pub async fn git_pull_branch(
    state: State<'_, AppState>,
    args: PullBranchArgs,
) -> AppResult<()> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::pull_branch(&ws.path, &args.branch).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffArgs {
    pub repo_id: String,
    pub base: String,
    pub target: String,
}

#[tauri::command]
pub async fn git_diff_branches(
    state: State<'_, AppState>,
    args: DiffArgs,
) -> AppResult<diff::BranchDiff> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::diff_branches(&ws.path, &args.base, &args.target).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffArgs {
    pub repo_id: String,
    pub base: String,
    pub target: String,
    pub path: String,
}

#[tauri::command]
pub async fn git_file_diff(
    state: State<'_, AppState>,
    args: FileDiffArgs,
) -> AppResult<diff::FileDiff> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::file_diff(&ws.path, &args.base, &args.target, &args.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContentArgs {
    pub repo_id: String,
    #[serde(rename = "ref")]
    pub r#ref: String,
    pub path: String,
}

#[tauri::command]
pub async fn git_file_content(
    state: State<'_, AppState>,
    args: FileContentArgs,
) -> AppResult<String> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::file_content(&ws.path, &args.r#ref, &args.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogArgs {
    pub repo_id: String,
    pub base: String,
    pub target: String,
    pub limit: Option<u32>,
}

#[tauri::command]
pub async fn git_log_between(
    state: State<'_, AppState>,
    args: LogArgs,
) -> AppResult<Vec<git_log::Commit>> {
    let ws = find_ws(&state, &args.repo_id)?;
    git_log::log_between(
        &ws.path,
        &args.base,
        &args.target,
        args.limit.unwrap_or(200),
    )
    .await
}

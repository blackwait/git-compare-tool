use crate::error::{AppError, AppResult};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: String,
}

pub async fn run(cwd: &str, args: &[&str], timeout_sec: u64) -> AppResult<GitOutput> {
    let mut cmd = Command::new("git");
    cmd.arg("--no-pager")
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("LC_ALL", "C.UTF-8")
        .env("GIT_TERMINAL_PROMPT", "0");

    let child = cmd.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AppError::GitNotFound,
        _ => AppError::from(e),
    })?;

    let out = match timeout(Duration::from_secs(timeout_sec), child.wait_with_output()).await {
        Err(_) => return Err(AppError::Timeout),
        Ok(r) => r.map_err(AppError::from)?,
    };

    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(AppError::GitFailed(msg));
    }

    Ok(GitOutput {
        stdout: out.stdout,
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    })
}

pub fn stdout_str(o: &GitOutput) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

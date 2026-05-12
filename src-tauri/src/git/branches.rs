use crate::error::AppResult;
use crate::git::runner::{run, stdout_str};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub kind: String, // "local" | "remote"
    pub is_head: bool,
    pub upstream: Option<String>,
}

pub async fn list(cwd: &str) -> AppResult<Vec<Branch>> {
    let out = run(
        cwd,
        &[
            "branch",
            "-a",
            "--format=%(HEAD)%00%(refname:short)%00%(upstream:short)",
        ],
        15,
    )
    .await?;
    let text = stdout_str(&out);
    let mut res = vec![];
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(3, '\u{0000}').collect();
        if parts.len() < 2 {
            continue;
        }
        let is_head = parts[0].trim() == "*";
        let name = parts[1].to_string();
        if name.is_empty() || name.starts_with("origin/HEAD") {
            continue;
        }
        let upstream = parts.get(2).and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });
        let kind = if name.contains('/') && !is_head {
            "remote"
        } else {
            "local"
        };
        res.push(Branch {
            name,
            kind: kind.into(),
            is_head,
            upstream,
        });
    }
    Ok(res)
}

pub async fn current(cwd: &str) -> AppResult<String> {
    let out = run(cwd, &["symbolic-ref", "--short", "HEAD"], 10).await?;
    Ok(stdout_str(&out).trim().to_string())
}

pub async fn fetch(cwd: &str, remote: Option<&str>) -> AppResult<()> {
    let args: Vec<&str> = match remote {
        Some(r) if !r.is_empty() => vec!["fetch", "--prune", r],
        _ => vec!["fetch", "--all", "--prune"],
    };
    run(cwd, &args, 120).await.map(|_| ())
}

/// fetch all + pull current branch (fast-forward only)
pub async fn fetch_and_pull(cwd: &str) -> AppResult<()> {
    run(cwd, &["fetch", "--all", "--prune"], 120).await?;
    run(cwd, &["pull", "--ff-only"], 60).await.map(|_| ())
}

/// 更新指定分支：如果是当前分支则 pull，否则 fetch 对应远程分支
pub async fn pull_branch(cwd: &str, branch: &str) -> AppResult<()> {
    // 先 fetch
    run(cwd, &["fetch", "--all", "--prune"], 120).await?;
    // 判断是否是当前分支
    let cur = current(cwd).await.unwrap_or_default();
    if cur == branch {
        // 当前分支直接 pull
        run(cwd, &["pull", "--ff-only"], 60).await.map(|_| ())
    } else {
        // 非当前分支，用 fetch 更新本地跟踪分支
        run(cwd, &["branch", "-f", branch, &format!("origin/{}", branch)], 30)
            .await
            .map(|_| ())
    }
}

pub async fn validate_repo(path: &str) -> AppResult<(bool, Option<String>)> {
    match run(path, &["rev-parse", "--show-toplevel"], 10).await {
        Ok(o) => Ok((
            true,
            Some(stdout_str(&o).trim().replace('\\', "/")),
        )),
        Err(_) => Ok((false, None)),
    }
}

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

pub async fn validate_repo(path: &str) -> AppResult<(bool, Option<String>)> {
    match run(path, &["rev-parse", "--show-toplevel"], 10).await {
        Ok(o) => Ok((
            true,
            Some(stdout_str(&o).trim().replace('\\', "/")),
        )),
        Err(_) => Ok((false, None)),
    }
}

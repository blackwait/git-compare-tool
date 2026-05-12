use crate::error::AppResult;
use crate::git::runner::{run, stdout_str};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub date: i64, // ms
    pub message: String,
}

pub async fn log_between(
    cwd: &str,
    base: &str,
    target: &str,
    limit: u32,
) -> AppResult<Vec<Commit>> {
    let range = format!("{base}..{target}");
    let max = format!("--max-count={}", limit);
    let fmt = "--format=%H|%h|%P|%an|%at|%s";
    let out = run(cwd, &["log", &range, &max, fmt], 20).await?;
    let mut res = vec![];
    for line in stdout_str(&out).lines() {
        let parts: Vec<&str> = line.splitn(6, '|').collect();
        if parts.len() < 6 {
            continue;
        }
        res.push(Commit {
            hash: parts[0].into(),
            short_hash: parts[1].into(),
            parents: parts[2]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            author: parts[3].into(),
            date: parts[4].parse::<i64>().unwrap_or(0) * 1000,
            message: parts[5].into(),
        });
    }
    Ok(res)
}

use crate::error::AppResult;
use crate::git::runner::{run, stdout_str};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub kind: String, // added | modified | deleted | renamed
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchDiff {
    pub base_ref: String,
    pub target_ref: String,
    pub files: Vec<FileChange>,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: String, // context | add | del
    pub content: String,
    pub old_line_no: Option<u64>,
    pub new_line_no: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: u64,
    pub old_lines: u64,
    pub new_start: u64,
    pub new_lines: u64,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub is_binary: bool,
    pub hunks: Vec<DiffHunk>,
    pub truncated: bool,
}

const MAX_FILES: usize = 5000;
const MAX_DIFF_LINES: usize = 10_000;

fn read_z_token(buf: &[u8], i: &mut usize) -> String {
    let end = buf[*i..]
        .iter()
        .position(|b| *b == 0)
        .map(|p| *i + p)
        .unwrap_or(buf.len());
    let s = String::from_utf8_lossy(&buf[*i..end]).to_string();
    *i = end + 1;
    s
}

pub async fn diff_branches(cwd: &str, base: &str, target: &str) -> AppResult<BranchDiff> {
    let range = format!("{base}..{target}");

    // name-status -z
    let ns = run(
        cwd,
        &["diff", "--name-status", "-z", &range],
        30,
    )
    .await?;
    let ns_bytes = &ns.stdout;
    let mut entries: Vec<(String, String, Option<String>)> = vec![];
    let mut i = 0usize;
    let mut truncated = false;
    while i < ns_bytes.len() {
        let status = read_z_token(ns_bytes, &mut i);
        if status.is_empty() {
            continue;
        }
        let first_ch = status.chars().next().unwrap_or(' ');
        if first_ch == 'R' || first_ch == 'C' {
            let old = read_z_token(ns_bytes, &mut i);
            let new = read_z_token(ns_bytes, &mut i);
            entries.push((status, new, Some(old)));
        } else {
            let p = read_z_token(ns_bytes, &mut i);
            if p.is_empty() {
                continue;
            }
            entries.push((status, p, None));
        }
        if entries.len() >= MAX_FILES {
            truncated = true;
            break;
        }
    }

    // numstat -z
    let nm = run(
        cwd,
        &["diff", "--numstat", "-z", &range],
        30,
    )
    .await?;
    let nm_bytes = &nm.stdout;
    let mut add_del: HashMap<String, (u64, u64)> = HashMap::new();
    let mut j = 0usize;
    while j < nm_bytes.len() {
        let rec = read_z_token(nm_bytes, &mut j);
        if rec.is_empty() {
            continue;
        }
        let mut parts = rec.split('\t');
        let add = parts.next().unwrap_or("0");
        let del = parts.next().unwrap_or("0");
        let path = parts.next().unwrap_or("").to_string();
        let a: u64 = add.parse().unwrap_or(0);
        let d: u64 = del.parse().unwrap_or(0);
        if path.is_empty() {
            // rename: two NUL-separated tokens follow (old then new)
            let _old = read_z_token(nm_bytes, &mut j);
            let new = read_z_token(nm_bytes, &mut j);
            if !new.is_empty() {
                add_del.insert(new, (a, d));
            }
        } else {
            add_del.insert(path, (a, d));
        }
    }

    let mut total_add = 0u64;
    let mut total_del = 0u64;
    let files: Vec<FileChange> = entries
        .into_iter()
        .map(|(status, path, old)| {
            let (a, d) = add_del.get(&path).cloned().unwrap_or((0, 0));
            total_add += a;
            total_del += d;
            let kind = match status.chars().next().unwrap_or(' ') {
                'A' => "added",
                'D' => "deleted",
                'R' | 'C' => "renamed",
                _ => "modified",
            }
            .to_string();
            FileChange {
                path,
                old_path: old,
                kind,
                additions: a,
                deletions: d,
            }
        })
        .collect();

    Ok(BranchDiff {
        base_ref: base.into(),
        target_ref: target.into(),
        files,
        total_additions: total_add,
        total_deletions: total_del,
        truncated,
    })
}

pub async fn file_diff(cwd: &str, base: &str, target: &str, path: &str) -> AppResult<FileDiff> {
    let range = format!("{base}..{target}");
    let out = run(
        cwd,
        &["diff", "--no-color", "-U3", &range, "--", path],
        30,
    )
    .await?;
    let text = stdout_str(&out);
    if text.contains("Binary files ") {
        return Ok(FileDiff {
            path: path.into(),
            old_path: None,
            is_binary: true,
            hunks: vec![],
            truncated: false,
        });
    }
    parse_unified(path, &text)
}

fn parse_unified(path: &str, text: &str) -> AppResult<FileDiff> {
    let mut hunks: Vec<DiffHunk> = vec![];
    let mut current: Option<DiffHunk> = None;
    let mut old_ln: u64 = 0;
    let mut new_ln: u64 = 0;
    let mut total_lines = 0usize;
    let mut truncated = false;

    for line in text.lines() {
        if line.starts_with("@@") {
            if let Some(h) = current.take() {
                hunks.push(h);
            }
            let mut segs = line.split(' ');
            // 跳过开头的 @@
            segs.next();
            let old_range = segs.next().unwrap_or("-0,0");
            let new_range = segs.next().unwrap_or("+0,0");
            let parse_range = |s: &str| -> (u64, u64) {
                let s = s.trim_start_matches('-').trim_start_matches('+');
                let mut it = s.split(',');
                let a: u64 = it.next().unwrap_or("0").parse().unwrap_or(0);
                let b: u64 = it.next().unwrap_or("1").parse().unwrap_or(1);
                (a, b)
            };
            let (oa, ob) = parse_range(old_range);
            let (na, nb) = parse_range(new_range);
            old_ln = oa;
            new_ln = na;
            current = Some(DiffHunk {
                old_start: oa,
                old_lines: ob,
                new_start: na,
                new_lines: nb,
                lines: vec![],
            });
        } else if let Some(h) = current.as_mut() {
            total_lines += 1;
            if total_lines > MAX_DIFF_LINES {
                truncated = true;
                break;
            }
            if let Some(rest) = line.strip_prefix('+') {
                if rest.starts_with("++") {
                    // 跳过 +++ b/path 这一行（实际 @@ 前会先出现）
                    continue;
                }
                h.lines.push(DiffLine {
                    kind: "add".into(),
                    content: rest.into(),
                    old_line_no: None,
                    new_line_no: Some(new_ln),
                });
                new_ln += 1;
            } else if let Some(rest) = line.strip_prefix('-') {
                if rest.starts_with("--") {
                    continue;
                }
                h.lines.push(DiffLine {
                    kind: "del".into(),
                    content: rest.into(),
                    old_line_no: Some(old_ln),
                    new_line_no: None,
                });
                old_ln += 1;
            } else if line.starts_with(' ') {
                h.lines.push(DiffLine {
                    kind: "context".into(),
                    content: line[1..].into(),
                    old_line_no: Some(old_ln),
                    new_line_no: Some(new_ln),
                });
                old_ln += 1;
                new_ln += 1;
            }
            // "\ No newline at end of file" 等忽略
        }
    }
    if let Some(h) = current {
        hunks.push(h);
    }
    Ok(FileDiff {
        path: path.into(),
        old_path: None,
        is_binary: false,
        hunks,
        truncated,
    })
}

pub async fn file_content(cwd: &str, r#ref: &str, path: &str) -> AppResult<String> {
    let obj = format!("{}:{}", r#ref, path);
    let out = run(cwd, &["show", &obj], 15).await?;
    Ok(stdout_str(&out))
}

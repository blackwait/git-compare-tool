# git-compare-tool Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
>
> **项目规则覆盖**：本仓库明确"不编写测试代码"。因此每个任务的验收不使用测试，而使用 **编译静态检查 + 手工验证脚本**。每个任务仍保持小颗粒 + 精确路径 + 完整代码 + 频繁提交。

**Goal:** 从零搭建一个 Tauri 2 + Vue 3 + TypeScript + Naive UI 的桌面端 Git 分支比对工具，支持多工作区管理、分支选择、Tree/List 双视图变更比对、只读代码与 diff 查看、base..target 提交图。

**Architecture:** 单体前端 + 薄后端。前端（Vue + Pinia + Naive UI）承担所有 UI 与状态；Rust 后端只暴露细粒度 Tauri 命令，通过 spawn 系统 `git` 命令执行查询并返回结构化 JSON；配置以 JSON 落到应用数据目录。

**Tech Stack:** Tauri 2.x · Vue 3 · TypeScript · Vite · Pinia · Naive UI · highlight.js · diff2html · Rust · tokio · serde · thiserror · tracing · ulid

**参考设计文档：** `docs/plans/2026-05-12-git-compare-tool-design.md`

---

## 阶段 P1：脚手架与基础依赖

### Task 1.1：初始化 Tauri + Vue + TS 项目

**Files:**
- Create: 整个 `git-compare-tool/` 目录结构（由脚手架生成）

**Step 1：使用官方脚手架**

Run（在当前工作区根目录下）：

```bash
pnpm create tauri-app@latest git-compare-tool --template vue-ts --manager pnpm
```

按提示一路选默认；app name / window title 都填 `git-compare-tool`。

**Step 2：验证启动**

Run：
```bash
cd git-compare-tool
pnpm install
pnpm tauri dev
```

Expected：Tauri 窗口能打开，显示默认 Vue 模板。按 Ctrl+C 关闭。

**Step 3：提交**

```bash
cd git-compare-tool && git init
git add -A
git commit -m "chore: 初始化 tauri + vue3 + ts 脚手架"
```

### Task 1.2：装 Naive UI 与 Pinia + Router

**Files:**
- Modify: `package.json`、`src/main.ts`

**Step 1：装依赖**

```bash
pnpm add naive-ui pinia vue-router@4
pnpm add -D @types/node vfonts
```

**Step 2：修改 `src/main.ts` 注册 Pinia + Router + Naive UI 全局消息**

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import 'vfonts/Lato.css';
import 'vfonts/FiraCode.css';

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');
```

**Step 3：新建路由占位**

Create `src/router/index.ts`：

```ts
import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'welcome', component: () => import('../views/Welcome.vue') },
  { path: '/repo/:id', name: 'repo', component: () => import('../views/RepoDetail.vue') },
  { path: '/settings', name: 'settings', component: () => import('../views/Settings.vue') },
];

export default createRouter({ history: createWebHashHistory(), routes });
```

Create `src/views/Welcome.vue`（先放一句欢迎）：

```vue
<template><n-card title="Git Compare Tool"><p>请先在左侧新建一个仓库工作区。</p></n-card></template>
<script setup lang="ts"></script>
```

（`RepoDetail.vue` / `Settings.vue` 先各放一个空 `<template><div/></template>`。）

**Step 4：改 `src/App.vue` 接入全局 Provider**

```vue
<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <router-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider, darkTheme, NCard } from 'naive-ui';
</script>
```

**Step 5：编译校验**

```bash
pnpm vue-tsc --noEmit
pnpm tauri dev    # 手工确认窗口打开显示 "Git Compare Tool"
```

Expected：无 TS 报错；窗口显示欢迎卡片。

**Step 6：提交**

```bash
git add -A && git commit -m "feat(p1): 接入 naive-ui / pinia / vue-router 基础骨架"
```

### Task 1.3：搭出前端类型、stores、api 目录空壳

**Files:**
- Create: `src/types/index.ts`
- Create: `src/stores/workspaces.ts`、`src/stores/repo.ts`、`src/stores/diff.ts`
- Create: `src/api/config.ts`、`src/api/git.ts`

**Step 1：类型定义**

`src/types/index.ts`（把设计文档 4.2 节所有 ts 类型原样落盘）：

```ts
export interface Workspace { id: string; name: string; path: string; remoteName?: string; createdAt: number; updatedAt: number; }
export interface Settings { theme: 'dark' | 'light' | 'auto'; defaultView: 'tree' | 'list'; }
export interface AppConfig { version: number; workspaces: Workspace[]; settings: Settings; }

export interface Branch { name: string; kind: 'local' | 'remote'; isHead: boolean; upstream?: string; }

export type ChangeKind = 'added' | 'modified' | 'deleted' | 'renamed';
export interface FileChange { path: string; oldPath?: string; kind: ChangeKind; additions: number; deletions: number; }
export interface BranchDiff { baseRef: string; targetRef: string; files: FileChange[]; totalAdditions: number; totalDeletions: number; truncated?: boolean; }

export interface DiffLine { kind: 'context' | 'add' | 'del'; content: string; oldLineNo?: number; newLineNo?: number; }
export interface DiffHunk { oldStart: number; oldLines: number; newStart: number; newLines: number; lines: DiffLine[]; }
export interface FileDiff { path: string; oldPath?: string; isBinary?: boolean; hunks: DiffHunk[]; truncated?: boolean; }

export interface Commit { hash: string; shortHash: string; parents: string[]; author: string; date: number; message: string; }
```

**Step 2：api 封装（先只签名，后端实现后直接可用）**

`src/api/config.ts`：

```ts
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, Workspace } from '../types';

export const configApi = {
  load: () => invoke<AppConfig>('config_load'),
  listWorkspaces: () => invoke<Workspace[]>('workspace_list'),
  createWorkspace: (args: { name: string; path: string; remoteName?: string }) =>
    invoke<Workspace>('workspace_create', { args }),
  updateWorkspace: (args: { id: string; patch: Partial<Workspace> }) =>
    invoke<Workspace>('workspace_update', { args }),
  deleteWorkspace: (id: string) => invoke<void>('workspace_delete', { args: { id } }),
  pickDir: () => invoke<string | null>('workspace_pick_dir'),
};
```

`src/api/git.ts`：

```ts
import { invoke } from '@tauri-apps/api/core';
import type { Branch, BranchDiff, FileDiff, Commit } from '../types';

export const gitApi = {
  validateRepo: (path: string) => invoke<{ valid: boolean; root?: string }>('git_validate_repo', { args: { path } }),
  listBranches: (repoId: string) => invoke<Branch[]>('git_list_branches', { args: { repoId } }),
  fetch: (repoId: string, remote?: string) => invoke<void>('git_fetch', { args: { repoId, remote } }),
  currentBranch: (repoId: string) => invoke<string>('git_current_branch', { args: { repoId } }),
  diffBranches: (repoId: string, base: string, target: string) =>
    invoke<BranchDiff>('git_diff_branches', { args: { repoId, base, target } }),
  fileDiff: (repoId: string, base: string, target: string, path: string) =>
    invoke<FileDiff>('git_file_diff', { args: { repoId, base, target, path } }),
  fileContent: (repoId: string, ref: string, path: string) =>
    invoke<string>('git_file_content', { args: { repoId, ref, path } }),
  logBetween: (repoId: string, base: string, target: string, limit?: number) =>
    invoke<Commit[]>('git_log_between', { args: { repoId, base, target, limit } }),
};
```

**Step 3：三个 Pinia store 占位**

`src/stores/workspaces.ts`：

```ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Workspace } from '../types';
import { configApi } from '../api/config';

export const useWorkspacesStore = defineStore('workspaces', () => {
  const list = ref<Workspace[]>([]);
  const loading = ref(false);

  async function reload() {
    loading.value = true;
    try { list.value = await configApi.listWorkspaces(); } finally { loading.value = false; }
  }
  return { list, loading, reload };
});
```

`src/stores/repo.ts`：

```ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Branch, Workspace } from '../types';

export const useRepoStore = defineStore('repo', () => {
  const current = ref<Workspace | null>(null);
  const branches = ref<Branch[]>([]);
  const base = ref<string | null>(null);
  const target = ref<string | null>(null);
  return { current, branches, base, target };
});
```

`src/stores/diff.ts`：

```ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { BranchDiff, FileDiff } from '../types';

export const useDiffStore = defineStore('diff', () => {
  const branchDiff = ref<BranchDiff | null>(null);
  const fileDiff = ref<FileDiff | null>(null);
  const viewMode = ref<'tree' | 'list'>('tree');
  return { branchDiff, fileDiff, viewMode };
});
```

**Step 4：编译校验 + 提交**

```bash
pnpm vue-tsc --noEmit
git add -A && git commit -m "feat(p1): 前端 types/api/stores 空壳"
```

---

## 阶段 P2：配置与工作区 CRUD

### Task 2.1：Rust 统一错误类型与依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/error.rs`

**Step 1：加 Rust 依赖**

`src-tauri/Cargo.toml` 的 `[dependencies]` 追加：

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["process", "time", "io-util", "macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
ulid = { version = "1", features = ["serde"] }
tauri-plugin-dialog = "2"
```

`src-tauri/tauri.conf.json` 的 `plugins` 下加：

```json
"dialog": {}
```

**Step 2：错误类型**

Create `src-tauri/src/error.rs`：

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("配置文件读写失败: {0}")]
    ConfigIo(String),
    #[error("路径不是 Git 仓库: {0}")]
    NotARepo(String),
    #[error("工作区不存在: {0}")]
    WorkspaceNotFound(String),
    #[error("未找到 git 可执行文件")]
    GitNotFound,
    #[error("Git 命令失败: {0}")]
    GitFailed(String),
    #[error("命令超时")]
    Timeout,
    #[error("参数错误: {0}")]
    InvalidArg(String),
    #[error("IO: {0}")]
    Io(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e.to_string()) }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self { AppError::ConfigIo(e.to_string()) }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
```

**Step 3：校验**

```bash
cd src-tauri && cargo check
```

Expected：无错误（可能有 unused warning，忽略）。

**Step 4：提交**

```bash
git add -A && git commit -m "feat(p2): rust 依赖与统一错误类型"
```

### Task 2.2：配置文件读写模块

**Files:**
- Create: `src-tauri/src/config/mod.rs`、`src-tauri/src/config/store.rs`

**Step 1：`src-tauri/src/config/mod.rs`**

```rust
pub mod store;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,        // "dark" | "light" | "auto"
    pub default_view: String, // "tree" | "list"
}

impl Default for Settings {
    fn default() -> Self {
        Self { theme: "dark".into(), default_view: "tree".into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub workspaces: Vec<Workspace>,
    pub settings: Settings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { version: 1, workspaces: vec![], settings: Settings::default() }
    }
}
```

**Step 2：`src-tauri/src/config/store.rs`（原子写入 + 损坏自愈）**

```rust
use super::AppConfig;
use crate::error::{AppError, AppResult};
use std::{fs, path::{Path, PathBuf}};

pub struct ConfigStore { pub path: PathBuf }

impl ConfigStore {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(AppError::from)?; }
        Ok(Self { path })
    }

    pub fn load(&self) -> AppResult<AppConfig> {
        if !self.path.exists() { return Ok(AppConfig::default()); }
        let raw = fs::read_to_string(&self.path).map_err(AppError::from)?;
        match serde_json::from_str::<AppConfig>(&raw) {
            Ok(c) => Ok(c),
            Err(e) => {
                // 损坏，备份再重置
                let bak = self.path.with_extension("json.bak");
                let _ = fs::copy(&self.path, &bak);
                let fresh = AppConfig::default();
                self.save(&fresh)?;
                tracing::warn!(?e, "config corrupted, reset to default, backup at {:?}", bak);
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

    pub fn mutate<F: FnOnce(&mut AppConfig) -> AppResult<()>>(&self, f: F) -> AppResult<AppConfig> {
        let mut cfg = self.load()?;
        f(&mut cfg)?;
        self.save(&cfg)?;
        Ok(cfg)
    }
}
```

**Step 3：校验**

```bash
cd src-tauri && cargo check
```

**Step 4：提交**

```bash
git add -A && git commit -m "feat(p2): 配置读写(原子写+损坏自愈)"
```

### Task 2.3：workspace CRUD 命令

**Files:**
- Create: `src-tauri/src/commands/mod.rs`、`src-tauri/src/commands/workspace.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1：`src-tauri/src/commands/mod.rs`**

```rust
pub mod workspace;

use crate::config::store::ConfigStore;
use std::sync::Mutex;

pub struct AppState { pub store: Mutex<ConfigStore> }
```

**Step 2：`src-tauri/src/commands/workspace.rs`**

```rust
use crate::config::{AppConfig, Workspace};
use crate::error::{AppError, AppResult};
use crate::commands::AppState;
use serde::Deserialize;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

fn now_ms() -> i64 {
    chrono_now_ms()
}
fn chrono_now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64).unwrap_or(0)
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
pub struct CreateArgs { pub name: String, pub path: String, pub remote_name: Option<String> }

#[tauri::command]
pub fn workspace_create(state: State<'_, AppState>, args: CreateArgs) -> AppResult<Workspace> {
    if args.name.trim().is_empty() { return Err(AppError::InvalidArg("name 不能为空".into())); }
    if args.path.trim().is_empty() { return Err(AppError::InvalidArg("path 不能为空".into())); }
    let ws = Workspace {
        id: format!("ws_{}", ulid::Ulid::new().to_string()),
        name: args.name, path: normalize(&args.path),
        remote_name: args.remote_name.or(Some("origin".into())),
        created_at: now_ms(), updated_at: now_ms(),
    };
    let cfg = state.store.lock().unwrap().mutate(|c| { c.workspaces.push(ws.clone()); Ok(()) })?;
    let saved = cfg.workspaces.into_iter().find(|w| w.id == ws.id).unwrap();
    Ok(saved)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArgs { pub id: String, pub patch: PatchWorkspace }

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchWorkspace {
    pub name: Option<String>, pub path: Option<String>, pub remote_name: Option<String>,
}

#[tauri::command]
pub fn workspace_update(state: State<'_, AppState>, args: UpdateArgs) -> AppResult<Workspace> {
    let cfg = state.store.lock().unwrap().mutate(|c| {
        let w = c.workspaces.iter_mut().find(|w| w.id == args.id)
            .ok_or_else(|| AppError::WorkspaceNotFound(args.id.clone()))?;
        if let Some(v) = args.patch.name { w.name = v; }
        if let Some(v) = args.patch.path { w.path = normalize(&v); }
        if let Some(v) = args.patch.remote_name { w.remote_name = Some(v); }
        w.updated_at = now_ms();
        Ok(())
    })?;
    Ok(cfg.workspaces.into_iter().find(|w| w.id == args.id).unwrap())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteArgs { pub id: String }

#[tauri::command]
pub fn workspace_delete(state: State<'_, AppState>, args: DeleteArgs) -> AppResult<()> {
    state.store.lock().unwrap().mutate(|c| {
        let before = c.workspaces.len();
        c.workspaces.retain(|w| w.id != args.id);
        if c.workspaces.len() == before { return Err(AppError::WorkspaceNotFound(args.id.clone())); }
        Ok(())
    })?;
    Ok(())
}

#[tauri::command]
pub async fn workspace_pick_dir(app: tauri::AppHandle) -> AppResult<Option<String>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |p| {
        let _ = tx.send(p.map(|p| p.to_string()));
    });
    Ok(rx.await.unwrap_or(None).map(|s| normalize(&s)))
}

fn normalize(p: &str) -> String { p.replace('\\', "/") }
```

**Step 3：`src-tauri/src/main.rs` 注册命令**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod config;
mod commands;

use commands::AppState;
use config::store::ConfigStore;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("appDataDir");
            let path = dir.join("git-compare-tool").join("config.json");
            let store = ConfigStore::new(path).expect("init config store");
            app.manage(AppState { store: Mutex::new(store) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::config_load,
            commands::workspace::workspace_list,
            commands::workspace::workspace_create,
            commands::workspace::workspace_update,
            commands::workspace::workspace_delete,
            commands::workspace::workspace_pick_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4：校验**

```bash
cd src-tauri && cargo check
```

**Step 5：提交**

```bash
git add -A && git commit -m "feat(p2): workspace CRUD tauri commands"
```

### Task 2.4：侧栏 + 新建/编辑 Modal（前端）

**Files:**
- Create: `src/components/WorkspaceSidebar.vue`、`src/components/WorkspaceEditModal.vue`
- Modify: `src/App.vue`、`src/views/Welcome.vue`、`src/views/RepoDetail.vue`

**Step 1：`src/components/WorkspaceEditModal.vue`**

```vue
<template>
  <n-modal :show="show" @update:show="$emit('update:show', $event)" preset="card"
           :title="workspace ? '编辑仓库' : '新建仓库'" style="width: 520px">
    <n-form :model="form" label-placement="left" label-width="auto">
      <n-form-item label="名称"><n-input v-model:value="form.name" placeholder="仓库显示名" /></n-form-item>
      <n-form-item label="路径">
        <n-input-group>
          <n-input v-model:value="form.path" placeholder="本地仓库绝对路径" />
          <n-button @click="onPick">选择...</n-button>
        </n-input-group>
      </n-form-item>
      <n-form-item label="远端名"><n-input v-model:value="form.remoteName" placeholder="origin" /></n-form-item>
    </n-form>
    <template #footer>
      <n-space justify="end">
        <n-button @click="$emit('update:show', false)">取消</n-button>
        <n-button type="primary" :loading="saving" @click="onSave">保存</n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { NModal, NForm, NFormItem, NInput, NInputGroup, NButton, NSpace, useMessage } from 'naive-ui';
import type { Workspace } from '../types';
import { configApi } from '../api/config';
import { gitApi } from '../api/git';

const props = defineProps<{ show: boolean; workspace?: Workspace | null }>();
const emit = defineEmits<{ (e: 'update:show', v: boolean): void; (e: 'saved', w: Workspace): void }>();

const message = useMessage();
const saving = ref(false);
const form = ref({ name: '', path: '', remoteName: 'origin' });

watch(() => props.show, (v) => {
  if (v) {
    form.value = props.workspace
      ? { name: props.workspace.name, path: props.workspace.path, remoteName: props.workspace.remoteName ?? 'origin' }
      : { name: '', path: '', remoteName: 'origin' };
  }
});

async function onPick() {
  const p = await configApi.pickDir();
  if (p) form.value.path = p;
}

async function onSave() {
  if (!form.value.name.trim() || !form.value.path.trim()) {
    message.error('名称与路径必填'); return;
  }
  saving.value = true;
  try {
    const v = await gitApi.validateRepo(form.value.path);
    if (!v.valid) { message.error('所选路径不是 git 仓库'); return; }
    const saved = props.workspace
      ? await configApi.updateWorkspace({ id: props.workspace.id, patch: form.value })
      : await configApi.createWorkspace(form.value);
    emit('saved', saved);
    emit('update:show', false);
    message.success('已保存');
  } catch (e: any) {
    message.error(e?.message ?? String(e));
  } finally { saving.value = false; }
}
</script>
```

**Step 2：`src/components/WorkspaceSidebar.vue`**

```vue
<template>
  <div class="sidebar">
    <n-space vertical size="small">
      <n-button block type="primary" @click="openCreate">+ 新建工作区</n-button>
      <n-list hoverable clickable>
        <n-list-item v-for="w in store.list" :key="w.id" @click="select(w.id)"
                     :class="{ active: route.params.id === w.id }">
          <n-thing :title="w.name">
            <template #description>
              <span class="path">{{ w.path }}</span>
            </template>
            <template #header-extra>
              <n-space :size="4">
                <n-button text size="tiny" @click.stop="openEdit(w)">编辑</n-button>
                <n-button text size="tiny" type="error" @click.stop="onDelete(w)">删除</n-button>
              </n-space>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-space>

    <WorkspaceEditModal v-model:show="modal" :workspace="editing" @saved="onSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { NSpace, NButton, NList, NListItem, NThing, useDialog, useMessage } from 'naive-ui';
import type { Workspace } from '../types';
import { useWorkspacesStore } from '../stores/workspaces';
import { configApi } from '../api/config';
import WorkspaceEditModal from './WorkspaceEditModal.vue';

const store = useWorkspacesStore();
const route = useRoute();
const router = useRouter();
const dialog = useDialog();
const message = useMessage();
const modal = ref(false);
const editing = ref<Workspace | null>(null);

onMounted(() => store.reload());

function openCreate() { editing.value = null; modal.value = true; }
function openEdit(w: Workspace) { editing.value = w; modal.value = true; }
async function onSaved() { await store.reload(); }

function select(id: string) { router.push({ name: 'repo', params: { id } }); }

function onDelete(w: Workspace) {
  dialog.warning({
    title: '删除确认', content: `删除工作区 "${w.name}"？不会影响本地仓库文件。`,
    positiveText: '删除', negativeText: '取消',
    onPositiveClick: async () => {
      await configApi.deleteWorkspace(w.id);
      await store.reload();
      message.success('已删除');
      if (route.params.id === w.id) router.push({ name: 'welcome' });
    }
  });
}
</script>

<style scoped>
.sidebar { width: 280px; padding: 12px; border-right: 1px solid #2a2a2a; height: 100vh; overflow-y: auto; }
.active { background: rgba(32,128,255,.1); }
.path { font-size: 12px; color: #888; word-break: break-all; }
</style>
```

**Step 3：更新 `src/App.vue` 变两栏布局**

```vue
<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider><n-dialog-provider><n-notification-provider>
      <div class="layout">
        <WorkspaceSidebar />
        <main class="main"><router-view /></main>
      </div>
    </n-notification-provider></n-dialog-provider></n-message-provider>
  </n-config-provider>
</template>
<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider, darkTheme } from 'naive-ui';
import WorkspaceSidebar from './components/WorkspaceSidebar.vue';
</script>
<style scoped>
.layout { display: flex; height: 100vh; }
.main { flex: 1; overflow: auto; padding: 16px; }
</style>
```

**Step 4：先给 git.ts 提供临时存根**（`git_validate_repo` 后端暂未实现）

修改 `src/api/git.ts`，暂时把 `validateRepo` 包成容错版：

```ts
validateRepo: async (path: string) => {
  try { return await invoke<{ valid: boolean; root?: string }>('git_validate_repo', { args: { path } }); }
  catch { return { valid: true }; }   // P3 实现后移除
},
```

**Step 5：手工验收**

```bash
pnpm tauri dev
```

Expected：左侧出现工作区列表，可新建（输入名称 + 选目录）、编辑、删除；配置落到 `%APPDATA%/<tauri-identifier>/git-compare-tool/config.json`。

**Step 6：提交**

```bash
git add -A && git commit -m "feat(p2): 侧栏 + 新建/编辑 Modal + 工作区 CRUD 前端接入"
```

---

## 阶段 P3：Git 基础（校验 / 分支 / fetch / 当前分支）

### Task 3.1：Git runner（spawn + 超时 + stderr 捕获）

**Files:**
- Create: `src-tauri/src/git/mod.rs`、`src-tauri/src/git/runner.rs`

**Step 1：`src-tauri/src/git/mod.rs`**

```rust
pub mod runner;
pub mod branches;
```

**Step 2：`src-tauri/src/git/runner.rs`**

```rust
use crate::error::{AppError, AppResult};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub struct GitOutput { pub stdout: Vec<u8>, pub stderr: String }

pub async fn run(cwd: &str, args: &[&str], timeout_sec: u64) -> AppResult<GitOutput> {
    let mut cmd = Command::new("git");
    cmd.arg("--no-pager").args(args).current_dir(cwd)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .env("LC_ALL", "C.UTF-8").env("GIT_TERMINAL_PROMPT", "0");
    let child = cmd.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AppError::GitNotFound,
        _ => AppError::from(e),
    })?;
    let out = match timeout(Duration::from_secs(timeout_sec), child.wait_with_output()).await {
        Err(_) => return Err(AppError::Timeout),
        Ok(r) => r.map_err(AppError::from)?,
    };
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(AppError::GitFailed(msg.trim().to_string()));
    }
    Ok(GitOutput { stdout: out.stdout, stderr: String::from_utf8_lossy(&out.stderr).to_string() })
}

pub fn stdout_str(o: &GitOutput) -> String { String::from_utf8_lossy(&o.stdout).to_string() }
```

**Step 3：校验**

```bash
cd src-tauri && cargo check
```

**Step 4：提交**

```bash
git add -A && git commit -m "feat(p3): git runner with timeout"
```

### Task 3.2：校验仓库 + 分支列表 + 当前分支 + fetch 命令

**Files:**
- Create: `src-tauri/src/git/branches.rs`
- Create: `src-tauri/src/commands/git.rs`
- Modify: `src-tauri/src/commands/mod.rs`、`src-tauri/src/main.rs`

**Step 1：`src-tauri/src/git/branches.rs`**

```rust
use crate::error::AppResult;
use crate::git::runner::{run, stdout_str};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Branch { pub name: String, pub kind: String, pub is_head: bool, pub upstream: Option<String> }

pub async fn list(cwd: &str) -> AppResult<Vec<Branch>> {
    let out = run(cwd, &[
        "branch", "-a",
        "--format=%(HEAD)%00%(refname:short)%00%(upstream:short)%00%(objecttype)"
    ], 15).await?;
    let text = stdout_str(&out);
    let mut res = vec![];
    for line in text.lines() {
        let parts: Vec<&str> = line.splitn(4, '\u{0000}').collect();
        if parts.len() < 2 { continue; }
        let is_head = parts[0].trim() == "*";
        let name = parts[1].to_string();
        if name.is_empty() || name.starts_with("origin/HEAD") { continue; }
        let upstream = parts.get(2).and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
        let kind = if name.contains('/') && !name.starts_with("HEAD") { "remote" } else { "local" };
        res.push(Branch { name, kind: kind.into(), is_head, upstream });
    }
    Ok(res)
}

pub async fn current(cwd: &str) -> AppResult<String> {
    let out = run(cwd, &["symbolic-ref", "--short", "HEAD"], 10).await?;
    Ok(stdout_str(&out).trim().to_string())
}

pub async fn fetch(cwd: &str, remote: Option<&str>) -> AppResult<()> {
    let r = remote.unwrap_or("--all");
    run(cwd, &["fetch", "--prune", r], 120).await.map(|_| ())
}

pub async fn validate_repo(path: &str) -> AppResult<(bool, Option<String>)> {
    match run(path, &["rev-parse", "--show-toplevel"], 10).await {
        Ok(o) => Ok((true, Some(stdout_str(&o).trim().replace('\\', "/")))),
        Err(_) => Ok((false, None)),
    }
}
```

**Step 2：`src-tauri/src/commands/git.rs`**

```rust
use crate::commands::AppState;
use crate::config::Workspace;
use crate::error::{AppError, AppResult};
use crate::git::branches;
use serde::{Deserialize, Serialize};
use tauri::State;

fn find_ws(state: &State<'_, AppState>, id: &str) -> AppResult<Workspace> {
    state.store.lock().unwrap().load()?.workspaces.into_iter()
        .find(|w| w.id == id).ok_or_else(|| AppError::WorkspaceNotFound(id.into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateArgs { pub path: String }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResult { pub valid: bool, pub root: Option<String> }

#[tauri::command]
pub async fn git_validate_repo(args: ValidateArgs) -> AppResult<ValidateResult> {
    let (valid, root) = branches::validate_repo(&args.path).await?;
    Ok(ValidateResult { valid, root })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoIdArgs { pub repo_id: String }

#[tauri::command]
pub async fn git_list_branches(state: State<'_, AppState>, args: RepoIdArgs) -> AppResult<Vec<branches::Branch>> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::list(&ws.path).await
}

#[tauri::command]
pub async fn git_current_branch(state: State<'_, AppState>, args: RepoIdArgs) -> AppResult<String> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::current(&ws.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchArgs { pub repo_id: String, pub remote: Option<String> }

#[tauri::command]
pub async fn git_fetch(state: State<'_, AppState>, args: FetchArgs) -> AppResult<()> {
    let ws = find_ws(&state, &args.repo_id)?;
    branches::fetch(&ws.path, args.remote.as_deref()).await
}
```

**Step 3：`src-tauri/src/commands/mod.rs` 追加**

```rust
pub mod git;
```

**Step 4：`src-tauri/src/main.rs` 新增 `mod git;` 并在 `invoke_handler!` 追加**

```rust
mod git;
// ...
commands::git::git_validate_repo,
commands::git::git_list_branches,
commands::git::git_current_branch,
commands::git::git_fetch,
```

**Step 5：移除 Task 2.4 中 `validateRepo` 的容错存根**（改回直接 invoke）

**Step 6：校验**

```bash
cd src-tauri && cargo check
pnpm vue-tsc --noEmit
```

**Step 7：提交**

```bash
git add -A && git commit -m "feat(p3): git validate/list-branches/current-branch/fetch"
```

### Task 3.3：分支选择器 + Repo 详情页框架

**Files:**
- Create: `src/components/BranchPicker.vue`
- Modify: `src/views/RepoDetail.vue`

**Step 1：`src/components/BranchPicker.vue`**

```vue
<template>
  <n-select :value="modelValue" :options="options" filterable
            :loading="loading" placeholder="选择分支"
            @update:value="$emit('update:modelValue', $event)" style="min-width: 240px" />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { NSelect } from 'naive-ui';
import type { Branch } from '../types';

const props = defineProps<{ branches: Branch[]; modelValue: string | null; loading?: boolean }>();
defineEmits<{ (e: 'update:modelValue', v: string): void }>();

const loading = computed(() => !!props.loading);
const options = computed(() => {
  const local = props.branches.filter(b => b.kind === 'local').map(b => ({
    label: (b.isHead ? '★ ' : '') + b.name, value: b.name,
  }));
  const remote = props.branches.filter(b => b.kind === 'remote').map(b => ({ label: b.name, value: b.name }));
  return [
    { type: 'group', label: '本地', key: 'local', children: local },
    { type: 'group', label: '远程', key: 'remote', children: remote },
  ];
});
</script>
```

**Step 2：`src/views/RepoDetail.vue`**

```vue
<template>
  <div v-if="repo.current" class="repo-detail">
    <n-space align="center" :size="12">
      <h2 style="margin:0">{{ repo.current.name }}</h2>
      <BranchPicker :branches="repo.branches" v-model="repo.base" :loading="loading" />
      <span>⇄</span>
      <BranchPicker :branches="repo.branches" v-model="repo.target" :loading="loading" />
      <n-button :loading="fetching" @click="onFetch">Fetch</n-button>
    </n-space>

    <n-alert v-if="error" type="error" style="margin-top:12px">{{ error }}</n-alert>

    <!-- P4 起此处渲染 diff 面板 -->
    <n-empty v-else style="margin-top:40px" description="选择两个分支后即可比对（P4 会实现）" />
  </div>
  <n-empty v-else description="请在左侧选择工作区" />
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { NSpace, NButton, NAlert, NEmpty, useMessage } from 'naive-ui';
import BranchPicker from '../components/BranchPicker.vue';
import { useRepoStore } from '../stores/repo';
import { useWorkspacesStore } from '../stores/workspaces';
import { gitApi } from '../api/git';

const route = useRoute();
const repo = useRepoStore();
const wsStore = useWorkspacesStore();
const message = useMessage();
const loading = ref(false);
const fetching = ref(false);
const error = ref<string | null>(null);

async function loadFor(id: string) {
  error.value = null;
  if (!wsStore.list.length) await wsStore.reload();
  repo.current = wsStore.list.find(w => w.id === id) ?? null;
  repo.branches = []; repo.base = null; repo.target = null;
  if (!repo.current) return;
  loading.value = true;
  try {
    repo.branches = await gitApi.listBranches(id);
    const cur = await gitApi.currentBranch(id);
    repo.base = cur;
    repo.target = repo.branches.find(b => b.kind === 'remote' && b.name.endsWith('/' + cur))?.name ?? null;
  } catch (e: any) { error.value = e?.message ?? String(e); }
  finally { loading.value = false; }
}

async function onFetch() {
  if (!repo.current) return;
  fetching.value = true;
  try { await gitApi.fetch(repo.current.id); message.success('fetch 完成'); await loadFor(repo.current.id); }
  catch (e: any) { message.error(e?.message ?? String(e)); }
  finally { fetching.value = false; }
}

watch(() => route.params.id as string, (id) => id && loadFor(id), { immediate: true });
onMounted(() => { const id = route.params.id as string; if (id) loadFor(id); });
</script>

<style scoped>
.repo-detail { padding: 4px; }
</style>
```

**Step 3：手工验收**

```bash
pnpm tauri dev
```

Expected：点击左侧仓库进入详情，上方两个分支选择器自动拉分支列表，可 fetch。

**Step 4：提交**

```bash
git add -A && git commit -m "feat(p3): 分支选择器 + Repo 详情页骨架"
```

---

## 阶段 P4：分支 Diff（文件变更 List 视图）

### Task 4.1：Rust 侧 diff_branches（name-status + numstat）

**Files:**
- Create: `src-tauri/src/git/diff.rs`
- Modify: `src-tauri/src/git/mod.rs`、`src-tauri/src/commands/git.rs`、`src-tauri/src/main.rs`

**Step 1：`src-tauri/src/git/mod.rs` 追加**

```rust
pub mod diff;
```

**Step 2：`src-tauri/src/git/diff.rs`**

```rust
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

const MAX_FILES: usize = 5000;

pub async fn diff_branches(cwd: &str, base: &str, target: &str) -> AppResult<BranchDiff> {
    // name-status -z
    let range = format!("{base}..{target}");
    let ns = run(cwd, &["diff", "--name-status", "-z", &range], 30).await?;
    let ns_bytes = &ns.stdout;
    let mut entries: Vec<(String, String, Option<String>)> = vec![];
    let mut i = 0;
    while i < ns_bytes.len() {
        let end = ns_bytes[i..].iter().position(|b| *b == 0).map(|p| i + p).unwrap_or(ns_bytes.len());
        let token = String::from_utf8_lossy(&ns_bytes[i..end]).to_string();
        i = end + 1;
        if token.is_empty() { continue; }
        let status_char = token.chars().next().unwrap_or(' ');
        let status = status_char.to_string();
        // 路径在下一个/下两个 token
        let read_path = |ni: &mut usize| -> String {
            let e = ns_bytes[*ni..].iter().position(|b| *b == 0).map(|p| *ni + p).unwrap_or(ns_bytes.len());
            let s = String::from_utf8_lossy(&ns_bytes[*ni..e]).to_string();
            *ni = e + 1;
            s
        };
        if status_char == 'R' || status_char == 'C' {
            let old = read_path(&mut i);
            let new = read_path(&mut i);
            entries.push((status, new, Some(old)));
        } else {
            let p = read_path(&mut i);
            entries.push((status, p, None));
        }
        if entries.len() >= MAX_FILES { break; }
    }

    // numstat -z
    let nm = run(cwd, &["diff", "--numstat", "-z", &range], 30).await?;
    let nm_bytes = &nm.stdout;
    let mut add_del: HashMap<String, (u64, u64)> = HashMap::new();
    let mut j = 0;
    while j < nm_bytes.len() {
        let end = nm_bytes[j..].iter().position(|b| *b == 0).map(|p| j + p).unwrap_or(nm_bytes.len());
        let rec = String::from_utf8_lossy(&nm_bytes[j..end]).to_string();
        j = end + 1;
        if rec.is_empty() { continue; }
        let mut parts = rec.split('\t');
        let add = parts.next().unwrap_or("0");
        let del = parts.next().unwrap_or("0");
        let path = parts.next().unwrap_or("").to_string();
        if path.is_empty() {
            // 重命名时 path 跟随两个 NUL 分隔的 token
            let _old = { let e = nm_bytes[j..].iter().position(|b| *b == 0).map(|p| j + p).unwrap_or(nm_bytes.len()); let s = String::from_utf8_lossy(&nm_bytes[j..e]).to_string(); j = e + 1; s };
            let new = { let e = nm_bytes[j..].iter().position(|b| *b == 0).map(|p| j + p).unwrap_or(nm_bytes.len()); let s = String::from_utf8_lossy(&nm_bytes[j..e]).to_string(); j = e + 1; s };
            add_del.insert(new, (parse_u(add), parse_u(del)));
        } else {
            add_del.insert(path, (parse_u(add), parse_u(del)));
        }
    }

    let mut total_add = 0u64;
    let mut total_del = 0u64;
    let files: Vec<FileChange> = entries.into_iter().map(|(status, path, old)| {
        let (a, d) = add_del.get(&path).cloned().unwrap_or((0, 0));
        total_add += a; total_del += d;
        let kind = match status.chars().next().unwrap_or(' ') {
            'A' => "added",
            'D' => "deleted",
            'R' | 'C' => "renamed",
            _   => "modified",
        }.to_string();
        FileChange { path, old_path: old, kind, additions: a, deletions: d }
    }).collect();

    Ok(BranchDiff {
        base_ref: base.into(), target_ref: target.into(),
        files, total_additions: total_add, total_deletions: total_del,
        truncated: false,
    })
}

fn parse_u(s: &str) -> u64 { s.parse().unwrap_or(0) }
```

**Step 3：在 `commands/git.rs` 新增命令**

```rust
use crate::git::diff;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffArgs { pub repo_id: String, pub base: String, pub target: String }

#[tauri::command]
pub async fn git_diff_branches(state: State<'_, AppState>, args: DiffArgs) -> AppResult<diff::BranchDiff> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::diff_branches(&ws.path, &args.base, &args.target).await
}
```

**Step 4：`main.rs` 的 `invoke_handler!` 追加 `commands::git::git_diff_branches,`**

**Step 5：校验**

```bash
cd src-tauri && cargo check
```

**Step 6：提交**

```bash
git add -A && git commit -m "feat(p4): rust git_diff_branches (name-status + numstat)"
```

### Task 4.2：前端 List 视图 + 变更面板

**Files:**
- Create: `src/components/FileChangeList.vue`
- Modify: `src/views/RepoDetail.vue`、`src/stores/diff.ts`

**Step 1：`diff.ts` 增加 load 方法**

```ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { BranchDiff, FileDiff } from '../types';
import { gitApi } from '../api/git';

export const useDiffStore = defineStore('diff', () => {
  const branchDiff = ref<BranchDiff | null>(null);
  const fileDiff = ref<FileDiff | null>(null);
  const viewMode = ref<'tree' | 'list'>('tree');
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadBranchDiff(repoId: string, base: string, target: string) {
    loading.value = true; error.value = null; branchDiff.value = null;
    try { branchDiff.value = await gitApi.diffBranches(repoId, base, target); }
    catch (e: any) { error.value = e?.message ?? String(e); }
    finally { loading.value = false; }
  }
  return { branchDiff, fileDiff, viewMode, loading, error, loadBranchDiff };
});
```

**Step 2：`src/components/FileChangeList.vue`**

```vue
<template>
  <div class="file-list">
    <div v-for="f in files" :key="f.path" class="row"
         :class="{ active: selected === f.path }" @click="$emit('select', f.path)">
      <span class="badge" :class="f.kind">{{ badge(f.kind) }}</span>
      <span class="path">{{ f.oldPath ? `${f.oldPath} → ${f.path}` : f.path }}</span>
      <span class="stat"><span class="add">+{{ f.additions }}</span> <span class="del">-{{ f.deletions }}</span></span>
    </div>
    <n-empty v-if="!files.length" description="没有差异" />
  </div>
</template>
<script setup lang="ts">
import { NEmpty } from 'naive-ui';
import type { FileChange } from '../types';
defineProps<{ files: FileChange[]; selected?: string | null }>();
defineEmits<{ (e: 'select', path: string): void }>();
function badge(k: FileChange['kind']) { return { added: 'A', modified: 'M', deleted: 'D', renamed: 'R' }[k]; }
</script>
<style scoped>
.file-list { height: 100%; overflow: auto; font-family: 'Fira Code', monospace; font-size: 12px; }
.row { display: flex; align-items: center; gap: 8px; padding: 4px 8px; cursor: pointer; border-bottom: 1px solid #1f1f1f; }
.row:hover, .row.active { background: rgba(32,128,255,.08); }
.badge { width: 18px; text-align: center; border-radius: 2px; font-weight: bold; }
.badge.added { background: #234d20; color: #6acf7c; }
.badge.modified { background: #4d4120; color: #e8c068; }
.badge.deleted { background: #4d2020; color: #ff6a6a; }
.badge.renamed { background: #20344d; color: #6aa9ff; }
.path { flex: 1; word-break: break-all; }
.stat .add { color: #6acf7c; } .stat .del { color: #ff6a6a; margin-left: 6px; }
</style>
```

**Step 3：更新 `RepoDetail.vue` 接入**

替换模板主体（保留上方分支选择器部分），增加变更面板与自动加载：

```vue
<template>
  <div v-if="repo.current" class="repo-detail">
    <n-space align="center" :size="12">
      <h2 style="margin:0">{{ repo.current.name }}</h2>
      <BranchPicker :branches="repo.branches" v-model="repo.base" :loading="loading" />
      <span>⇄</span>
      <BranchPicker :branches="repo.branches" v-model="repo.target" :loading="loading" />
      <n-button :loading="fetching" @click="onFetch">Fetch</n-button>
      <n-button @click="reloadDiff" :loading="diff.loading">比对</n-button>
    </n-space>

    <n-alert v-if="error" type="error" style="margin-top:8px">{{ error }}</n-alert>
    <n-alert v-if="diff.error" type="error" style="margin-top:8px">{{ diff.error }}</n-alert>

    <div class="diff-area" v-if="diff.branchDiff">
      <div class="stat-bar">
        共 {{ diff.branchDiff.files.length }} 个文件
        <span class="add">+{{ diff.branchDiff.totalAdditions }}</span>
        <span class="del">-{{ diff.branchDiff.totalDeletions }}</span>
      </div>
      <FileChangeList :files="diff.branchDiff.files" @select="onSelectFile" :selected="selectedPath" />
    </div>
  </div>
  <n-empty v-else description="请在左侧选择工作区" />
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { NSpace, NButton, NAlert, NEmpty, useMessage } from 'naive-ui';
import BranchPicker from '../components/BranchPicker.vue';
import FileChangeList from '../components/FileChangeList.vue';
import { useRepoStore } from '../stores/repo';
import { useWorkspacesStore } from '../stores/workspaces';
import { useDiffStore } from '../stores/diff';
import { gitApi } from '../api/git';

const route = useRoute();
const repo = useRepoStore();
const wsStore = useWorkspacesStore();
const diff = useDiffStore();
const message = useMessage();
const loading = ref(false);
const fetching = ref(false);
const error = ref<string | null>(null);
const selectedPath = ref<string | null>(null);

async function loadFor(id: string) {
  error.value = null; diff.branchDiff = null; diff.fileDiff = null; selectedPath.value = null;
  if (!wsStore.list.length) await wsStore.reload();
  repo.current = wsStore.list.find(w => w.id === id) ?? null;
  repo.branches = []; repo.base = null; repo.target = null;
  if (!repo.current) return;
  loading.value = true;
  try {
    repo.branches = await gitApi.listBranches(id);
    const cur = await gitApi.currentBranch(id);
    repo.base = cur;
    repo.target = repo.branches.find(b => b.kind === 'remote' && b.name.endsWith('/' + cur))?.name ?? null;
  } catch (e: any) { error.value = e?.message ?? String(e); }
  finally { loading.value = false; }
}

async function reloadDiff() {
  if (!repo.current || !repo.base || !repo.target) { message.warning('请选择两个分支'); return; }
  await diff.loadBranchDiff(repo.current.id, repo.base, repo.target);
}

async function onFetch() {
  if (!repo.current) return;
  fetching.value = true;
  try { await gitApi.fetch(repo.current.id); message.success('fetch 完成'); await loadFor(repo.current.id); }
  catch (e: any) { message.error(e?.message ?? String(e)); }
  finally { fetching.value = false; }
}

function onSelectFile(p: string) { selectedPath.value = p; /* P6 用 */ }

watch(() => route.params.id as string, (id) => id && loadFor(id), { immediate: true });
watch([() => repo.base, () => repo.target], () => { if (repo.base && repo.target) reloadDiff(); });
onMounted(() => { const id = route.params.id as string; if (id) loadFor(id); });
</script>

<style scoped>
.repo-detail { display: flex; flex-direction: column; height: 100%; gap: 8px; }
.diff-area { flex: 1; display: flex; flex-direction: column; border: 1px solid #2a2a2a; border-radius: 4px; }
.stat-bar { padding: 6px 10px; border-bottom: 1px solid #2a2a2a; font-size: 12px; }
.stat-bar .add { color: #6acf7c; margin-left: 8px; } .stat-bar .del { color: #ff6a6a; margin-left: 6px; }
</style>
```

**Step 4：手工验收**

```bash
pnpm tauri dev
```

Expected：选工作区 → 选两个分支 → 下方出现文件变更列表，带 A/M/D/R 图标和 +/- 行数。

**Step 5：提交**

```bash
git add -A && git commit -m "feat(p4): 文件变更 list 视图 + 自动比对"
```

---

## 阶段 P5：Tree 视图 + 视图切换

### Task 5.1：FileChangeTree 组件

**Files:**
- Create: `src/components/FileChangeTree.vue`
- Modify: `src/views/RepoDetail.vue`

**Step 1：`src/components/FileChangeTree.vue`**

```vue
<template>
  <n-tree
    block-line block-node :data="treeData" :key-field="'key'" :label-field="'label'"
    :selected-keys="selected ? [selected] : []"
    @update:selected-keys="onSelect" :virtual-scroll="true"
    style="height: 100%"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NTree, type TreeOption } from 'naive-ui';
import type { FileChange } from '../types';

const props = defineProps<{ files: FileChange[]; selected?: string | null }>();
const emit = defineEmits<{ (e: 'select', path: string): void }>();

const treeData = computed<TreeOption[]>(() => {
  const root: any = { children: {} };
  for (const f of props.files) {
    const segs = f.path.split('/');
    let cur = root;
    segs.forEach((s, i) => {
      if (i === segs.length - 1) {
        cur.children[s] = { key: f.path, label: `${s}  +${f.additions} -${f.deletions}`, leaf: true, _file: f };
      } else {
        cur.children[s] = cur.children[s] || { key: segs.slice(0, i + 1).join('/'), label: s, children: {} };
        cur = cur.children[s];
      }
    });
  }
  const build = (node: any): TreeOption[] => Object.entries(node.children).map(([name, child]: any) => {
    if (child.leaf) return child;
    return { key: child.key, label: child.label, children: build(child) };
  });
  return build(root);
});

function onSelect(keys: string[]) {
  const k = keys[0]; if (!k) return;
  const hit = props.files.find(f => f.path === k);
  if (hit) emit('select', hit.path);
}
</script>
```

**Step 2：`RepoDetail.vue` 加视图切换**

在分支选择器 `n-space` 内追加：

```vue
<n-button-group>
  <n-button :type="diff.viewMode === 'tree' ? 'primary' : 'default'" @click="diff.viewMode = 'tree'">Tree</n-button>
  <n-button :type="diff.viewMode === 'list' ? 'primary' : 'default'" @click="diff.viewMode = 'list'">List</n-button>
</n-button-group>
```

把 `FileChangeList` 所在位置改为条件渲染：

```vue
<FileChangeTree v-if="diff.viewMode === 'tree'" :files="diff.branchDiff.files" @select="onSelectFile" :selected="selectedPath" />
<FileChangeList v-else :files="diff.branchDiff.files" @select="onSelectFile" :selected="selectedPath" />
```

在 `<script setup>` 顶部 `import FileChangeTree from '../components/FileChangeTree.vue';` 和 `NButtonGroup`。

**Step 3：手工验收 + 提交**

```bash
pnpm tauri dev    # 手工切 Tree/List
git add -A && git commit -m "feat(p5): 文件变更 tree 视图 + 视图切换"
```

---

## 阶段 P6：文件 Diff 与源码查看

### Task 6.1：Rust 侧 file_diff + file_content

**Files:**
- Modify: `src-tauri/src/git/diff.rs`、`src-tauri/src/commands/git.rs`、`src-tauri/src/main.rs`

**Step 1：`diff.rs` 追加 parse + 命令用的结构与函数**

```rust
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine { pub kind: String, pub content: String, pub old_line_no: Option<u64>, pub new_line_no: Option<u64> }

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk { pub old_start: u64, pub old_lines: u64, pub new_start: u64, pub new_lines: u64, pub lines: Vec<DiffLine> }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub is_binary: bool,
    pub hunks: Vec<DiffHunk>,
    pub truncated: bool,
}

const MAX_DIFF_LINES: usize = 10_000;

pub async fn file_diff(cwd: &str, base: &str, target: &str, path: &str) -> AppResult<FileDiff> {
    let range = format!("{base}..{target}");
    let out = run(cwd, &["diff", "--no-color", "-U3", &range, "--", path], 30).await?;
    let text = stdout_str(&out);
    if text.contains("Binary files ") {
        return Ok(FileDiff { path: path.into(), old_path: None, is_binary: true, hunks: vec![], truncated: false });
    }
    parse_unified(path, &text)
}

fn parse_unified(path: &str, text: &str) -> AppResult<FileDiff> {
    let mut hunks: Vec<DiffHunk> = vec![];
    let mut current: Option<DiffHunk> = None;
    let mut old_ln: u64 = 0; let mut new_ln: u64 = 0;
    let mut total_lines = 0usize;
    let mut truncated = false;

    for line in text.lines() {
        if line.starts_with("@@") {
            if let Some(h) = current.take() { hunks.push(h); }
            // 解析 @@ -a,b +c,d @@
            let inner = line.trim_start_matches("@@").trim_end_matches("@@").trim();
            let parts: Vec<&str> = inner.split_whitespace().collect();
            let parse_range = |s: &str| -> (u64, u64) {
                let s = s.trim_start_matches('-').trim_start_matches('+');
                let mut it = s.split(',');
                let a: u64 = it.next().unwrap_or("0").parse().unwrap_or(0);
                let b: u64 = it.next().unwrap_or("1").parse().unwrap_or(1);
                (a, b)
            };
            if parts.len() >= 2 {
                let (oa, ob) = parse_range(parts[0]);
                let (na, nb) = parse_range(parts[1]);
                old_ln = oa; new_ln = na;
                current = Some(DiffHunk { old_start: oa, old_lines: ob, new_start: na, new_lines: nb, lines: vec![] });
            }
        } else if let Some(h) = current.as_mut() {
            total_lines += 1;
            if total_lines > MAX_DIFF_LINES { truncated = true; break; }
            if let Some(rest) = line.strip_prefix('+') {
                h.lines.push(DiffLine { kind: "add".into(), content: rest.into(), old_line_no: None, new_line_no: Some(new_ln) });
                new_ln += 1;
            } else if let Some(rest) = line.strip_prefix('-') {
                h.lines.push(DiffLine { kind: "del".into(), content: rest.into(), old_line_no: Some(old_ln), new_line_no: None });
                old_ln += 1;
            } else if line.starts_with(' ') {
                h.lines.push(DiffLine { kind: "context".into(), content: line[1..].into(), old_line_no: Some(old_ln), new_line_no: Some(new_ln) });
                old_ln += 1; new_ln += 1;
            } // 忽略 \ No newline at end of file
        }
    }
    if let Some(h) = current { hunks.push(h); }
    Ok(FileDiff { path: path.into(), old_path: None, is_binary: false, hunks, truncated })
}

pub async fn file_content(cwd: &str, r#ref: &str, path: &str) -> AppResult<String> {
    let obj = format!("{}:{}", r#ref, path);
    let out = run(cwd, &["show", &obj], 15).await?;
    Ok(stdout_str(&out))
}
```

**Step 2：`commands/git.rs` 新增命令**

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffArgs { pub repo_id: String, pub base: String, pub target: String, pub path: String }

#[tauri::command]
pub async fn git_file_diff(state: State<'_, AppState>, args: FileDiffArgs) -> AppResult<diff::FileDiff> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::file_diff(&ws.path, &args.base, &args.target, &args.path).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContentArgs { pub repo_id: String, pub r#ref: String, pub path: String }

#[tauri::command]
pub async fn git_file_content(state: State<'_, AppState>, args: FileContentArgs) -> AppResult<String> {
    let ws = find_ws(&state, &args.repo_id)?;
    diff::file_content(&ws.path, &args.r#ref, &args.path).await
}
```

**Step 3：`main.rs` 追加 handler**

```rust
commands::git::git_file_diff,
commands::git::git_file_content,
```

**Step 4：校验 + 提交**

```bash
cd src-tauri && cargo check
git add -A && git commit -m "feat(p6): rust file_diff + file_content"
```

### Task 6.2：DiffViewer 组件 + 右侧面板

**Files:**
- Create: `src/components/DiffViewer.vue`
- Modify: `src/views/RepoDetail.vue`、`src/stores/diff.ts`

**Step 1：装 highlight.js**

```bash
pnpm add highlight.js
```

**Step 2：`diff.ts` 增加 loadFileDiff / loadFileContent**

```ts
async function loadFileDiff(repoId: string, base: string, target: string, path: string) {
  fileDiff.value = null;
  fileDiff.value = await gitApi.fileDiff(repoId, base, target, path);
}
```

（返回值里 export：`loadFileDiff`）

**Step 3：`src/components/DiffViewer.vue`**

```vue
<template>
  <n-tabs type="line" v-model:value="tab">
    <n-tab-pane name="diff" tab="Diff">
      <div v-if="fileDiff?.isBinary" class="tip">二进制文件，无法预览</div>
      <div v-else-if="!fileDiff || !fileDiff.hunks.length" class="tip">未选择文件或无差异</div>
      <div v-else class="diff">
        <div v-for="(h, hi) in fileDiff.hunks" :key="hi" class="hunk">
          <div class="hunk-head">@@ -{{ h.oldStart }},{{ h.oldLines }} +{{ h.newStart }},{{ h.newLines }} @@</div>
          <div v-for="(l, li) in h.lines" :key="li" class="line" :class="l.kind">
            <span class="ln">{{ l.oldLineNo ?? '' }}</span>
            <span class="ln">{{ l.newLineNo ?? '' }}</span>
            <pre class="content">{{ prefix(l.kind) + l.content }}</pre>
          </div>
        </div>
        <div v-if="fileDiff.truncated" class="tip">结果过大，已截断</div>
      </div>
    </n-tab-pane>
    <n-tab-pane name="source" tab="源文件 (目标分支)">
      <div v-if="!source" class="tip">点击"加载源文件"</div>
      <pre v-else class="source" v-html="highlighted"></pre>
      <n-button v-if="path && !source" @click="loadSource" size="small">加载源文件</n-button>
    </n-tab-pane>
  </n-tabs>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { NTabs, NTabPane, NButton } from 'naive-ui';
import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';
import type { FileDiff } from '../types';
import { gitApi } from '../api/git';

const props = defineProps<{ repoId: string | null; targetRef: string | null; path: string | null; fileDiff: FileDiff | null }>();
const tab = ref<'diff' | 'source'>('diff');
const source = ref<string | null>(null);

function prefix(k: string) { return k === 'add' ? '+' : k === 'del' ? '-' : ' '; }

const highlighted = computed(() => {
  if (!source.value) return '';
  try { return hljs.highlightAuto(source.value).value; } catch { return source.value; }
});

async function loadSource() {
  if (!props.repoId || !props.targetRef || !props.path) return;
  source.value = await gitApi.fileContent(props.repoId, props.targetRef, props.path);
}

watch(() => props.path, () => { source.value = null; tab.value = 'diff'; });
</script>

<style scoped>
.tip { padding: 12px; color: #888; font-size: 12px; }
.diff { font-family: 'Fira Code', monospace; font-size: 12px; }
.hunk-head { background: #1a2332; color: #9ec5ff; padding: 2px 8px; }
.line { display: grid; grid-template-columns: 50px 50px 1fr; }
.line .ln { color: #666; text-align: right; padding: 0 8px; background: #151515; }
.line .content { margin: 0; white-space: pre-wrap; }
.line.add { background: rgba(80,200,120,.08); } .line.add .content { color: #b0e6b0; }
.line.del { background: rgba(255,80,80,.08); } .line.del .content { color: #e0a0a0; }
.source { padding: 8px 12px; font-size: 12px; white-space: pre-wrap; }
</style>
```

**Step 4：`RepoDetail.vue` 接入 + 改为左右分栏**

将 `diff-area` 结构改成左右两栏：

```vue
<div class="diff-area" v-if="diff.branchDiff">
  <div class="left">
    <div class="stat-bar">...</div>
    <FileChangeTree v-if="diff.viewMode === 'tree'" ... />
    <FileChangeList v-else ... />
  </div>
  <div class="right">
    <DiffViewer :repo-id="repo.current?.id ?? null" :target-ref="repo.target"
                :path="selectedPath" :file-diff="diff.fileDiff" />
  </div>
</div>
```

`onSelectFile` 里加载 fileDiff：

```ts
async function onSelectFile(p: string) {
  selectedPath.value = p;
  if (repo.current && repo.base && repo.target) {
    await diff.loadFileDiff(repo.current.id, repo.base, repo.target, p);
  }
}
```

样式：

```css
.diff-area { flex: 1; display: grid; grid-template-columns: 40% 60%; gap: 8px; }
.left, .right { border: 1px solid #2a2a2a; border-radius: 4px; overflow: auto; }
```

**Step 5：手工验收**

```bash
pnpm tauri dev
```

Expected：点左侧文件，右侧显示 unified diff；切源文件标签能显示高亮源码。

**Step 6：提交**

```bash
git add -A && git commit -m "feat(p6): DiffViewer 组件 + 左右分栏"
```

---

## 阶段 P7：提交图（base..target）

### Task 7.1：Rust log_between

**Files:**
- Create: `src-tauri/src/git/log.rs`
- Modify: `src-tauri/src/git/mod.rs`、`src-tauri/src/commands/git.rs`、`src-tauri/src/main.rs`

**Step 1：`src-tauri/src/git/mod.rs` 追加 `pub mod log;`**

**Step 2：`src-tauri/src/git/log.rs`**

```rust
use crate::error::AppResult;
use crate::git::runner::{run, stdout_str};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub hash: String, pub short_hash: String,
    pub parents: Vec<String>, pub author: String, pub date: i64, pub message: String,
}

pub async fn log_between(cwd: &str, base: &str, target: &str, limit: u32) -> AppResult<Vec<Commit>> {
    let range = format!("{base}..{target}");
    let fmt = "%H|%h|%P|%an|%at|%s";
    let out = run(cwd, &["log", &range, &format!("--max-count={}", limit), &format!("--format={}", fmt)], 20).await?;
    let mut res = vec![];
    for line in stdout_str(&out).lines() {
        let parts: Vec<&str> = line.splitn(6, '|').collect();
        if parts.len() < 6 { continue; }
        res.push(Commit {
            hash: parts[0].into(), short_hash: parts[1].into(),
            parents: parts[2].split_whitespace().map(|s| s.to_string()).collect(),
            author: parts[3].into(),
            date: parts[4].parse::<i64>().unwrap_or(0) * 1000,
            message: parts[5].into(),
        });
    }
    Ok(res)
}
```

**Step 3：命令**

`commands/git.rs`：

```rust
use crate::git::log as git_log;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogArgs { pub repo_id: String, pub base: String, pub target: String, pub limit: Option<u32> }

#[tauri::command]
pub async fn git_log_between(state: State<'_, AppState>, args: LogArgs) -> AppResult<Vec<git_log::Commit>> {
    let ws = find_ws(&state, &args.repo_id)?;
    git_log::log_between(&ws.path, &args.base, &args.target, args.limit.unwrap_or(200)).await
}
```

`main.rs` 追加 `commands::git::git_log_between,`。

**Step 4：校验 + 提交**

```bash
cd src-tauri && cargo check
git add -A && git commit -m "feat(p7): rust log_between"
```

### Task 7.2：CommitGraph 组件（SVG 简图）

**Files:**
- Create: `src/components/CommitGraph.vue`
- Modify: `src/views/RepoDetail.vue`

**Step 1：`src/components/CommitGraph.vue`**

```vue
<template>
  <div class="graph-wrap">
    <div class="hint" v-if="!commits.length">无提交</div>
    <svg v-else :width="400" :height="commits.length * 26 + 10" class="graph">
      <g v-for="(c, i) in commits" :key="c.hash" :transform="`translate(10, ${i * 26 + 14})`">
        <line v-if="i < commits.length - 1" x1="0" y1="0" x2="0" y2="26" stroke="#555" />
        <circle r="5" :fill="c.parents.length > 1 ? '#e8c068' : '#6aa9ff'" />
        <text x="16" y="4" fill="#ddd" font-size="12">
          {{ c.shortHash }} {{ c.message.slice(0, 60) }}
        </text>
      </g>
    </svg>
  </div>
</template>
<script setup lang="ts">
import type { Commit } from '../types';
defineProps<{ commits: Commit[] }>();
</script>
<style scoped>
.graph-wrap { padding: 8px; overflow: auto; max-height: 220px; border-top: 1px solid #2a2a2a; }
.hint { color: #888; font-size: 12px; }
</style>
```

**Step 2：`RepoDetail.vue` 底部加提交图**

```ts
import CommitGraph from '../components/CommitGraph.vue';
import type { Commit } from '../types';
const commits = ref<Commit[]>([]);
async function loadCommits() {
  if (!repo.current || !repo.base || !repo.target) { commits.value = []; return; }
  commits.value = await gitApi.logBetween(repo.current.id, repo.base, repo.target, 200);
}
watch([() => repo.base, () => repo.target], () => loadCommits());
```

模板里 `diff-area` 后追加：

```vue
<CommitGraph v-if="diff.branchDiff" :commits="commits" />
```

**Step 3：手工验收 + 提交**

```bash
pnpm tauri dev
git add -A && git commit -m "feat(p7): CommitGraph SVG 组件"
```

---

## 阶段 P8：顶部菜单 + 设置页

### Task 8.1：顶部菜单

**Files:**
- Create: `src/components/AppMenu.vue`
- Modify: `src/App.vue`

**Step 1：`src/components/AppMenu.vue`**

```vue
<template>
  <n-menu mode="horizontal" :options="options" @update:value="onClick" />
</template>
<script setup lang="ts">
import { NMenu, type MenuOption } from 'naive-ui';
import { useRouter } from 'vue-router';
import { useWorkspacesStore } from '../stores/workspaces';

const router = useRouter();
const store = useWorkspacesStore();

const options: MenuOption[] = [
  { label: '仓库', key: 'repo', children: [
    { label: '+ 新建工作区', key: 'new' },
    { label: '刷新列表', key: 'reload' },
  ]},
  { label: '设置', key: 'settings' },
  { label: '帮助', key: 'help' },
];
async function onClick(key: string) {
  if (key === 'new') window.dispatchEvent(new Event('gct:open-create'));
  if (key === 'reload') store.reload();
  if (key === 'settings') router.push({ name: 'settings' });
  if (key === 'help') router.push({ name: 'welcome' });
}
</script>
```

**Step 2：`WorkspaceSidebar.vue` 监听事件打开 Modal**

```ts
import { onMounted, onUnmounted } from 'vue';
const openFromMenu = () => openCreate();
onMounted(() => window.addEventListener('gct:open-create', openFromMenu));
onUnmounted(() => window.removeEventListener('gct:open-create', openFromMenu));
```

**Step 3：`App.vue` 挂载菜单**

```vue
<div class="layout">
  <div class="top"><AppMenu /></div>
  <div class="body">
    <WorkspaceSidebar />
    <main class="main"><router-view /></main>
  </div>
</div>
```

样式调整为 `.layout { display: flex; flex-direction: column; height: 100vh; } .body { flex: 1; display: flex; overflow: hidden; }`。

**Step 4：提交**

```bash
git add -A && git commit -m "feat(p8): 顶部菜单"
```

### Task 8.2：设置页（主题 + 默认视图）

**Files:**
- Modify: `src/views/Settings.vue`、`src/stores/diff.ts`、`src/App.vue`
- Create: `src/stores/settings.ts`

**Step 1：`src/stores/settings.ts`**

```ts
import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { Settings } from '../types';
import { configApi } from '../api/config';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light' | 'auto'>('dark');
  const defaultView = ref<'tree' | 'list'>('tree');

  async function load() {
    const cfg = await configApi.load();
    theme.value = cfg.settings.theme as any;
    defaultView.value = cfg.settings.defaultView as any;
  }
  async function save() {
    await invoke('settings_save', { args: { theme: theme.value, defaultView: defaultView.value } });
  }
  watch([theme, defaultView], save);
  return { theme, defaultView, load };
});
```

**Step 2：Rust 侧加 `settings_save` 命令**

在 `commands/workspace.rs` 追加：

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsArgs { pub theme: String, pub default_view: String }

#[tauri::command]
pub fn settings_save(state: State<'_, AppState>, args: SettingsArgs) -> AppResult<()> {
    state.store.lock().unwrap().mutate(|c| {
        c.settings.theme = args.theme;
        c.settings.default_view = args.default_view;
        Ok(())
    })?;
    Ok(())
}
```

`main.rs` 追加 handler：`commands::workspace::settings_save,`。

**Step 3：`Settings.vue`**

```vue
<template>
  <n-card title="设置" style="max-width: 480px">
    <n-form label-placement="left" label-width="auto">
      <n-form-item label="主题">
        <n-radio-group v-model:value="s.theme">
          <n-radio value="dark">深色</n-radio>
          <n-radio value="light">浅色</n-radio>
          <n-radio value="auto">跟随系统</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item label="默认视图">
        <n-radio-group v-model:value="s.defaultView">
          <n-radio value="tree">Tree</n-radio>
          <n-radio value="list">List</n-radio>
        </n-radio-group>
      </n-form-item>
    </n-form>
  </n-card>
</template>
<script setup lang="ts">
import { onMounted } from 'vue';
import { NCard, NForm, NFormItem, NRadioGroup, NRadio } from 'naive-ui';
import { useSettingsStore } from '../stores/settings';
const s = useSettingsStore();
onMounted(() => s.load());
</script>
```

**Step 4：`App.vue` 根据 theme 切换**

```vue
<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { darkTheme } from 'naive-ui';
import { useSettingsStore } from './stores/settings';
import { useDiffStore } from './stores/diff';
const s = useSettingsStore();
const diff = useDiffStore();
const theme = computed(() => s.theme === 'light' ? null : darkTheme);
onMounted(async () => { await s.load(); diff.viewMode = s.defaultView; });
</script>
<!-- <n-config-provider :theme="theme"> -->
```

**Step 5：校验 + 提交**

```bash
cd src-tauri && cargo check
pnpm vue-tsc --noEmit
git add -A && git commit -m "feat(p8): 设置页 + 主题与默认视图持久化"
```

---

## 阶段 P9：打磨

### Task 9.1：启动探测 git + 失效工作区徽章

**Files:**
- Modify: `src-tauri/src/main.rs`、`src/components/WorkspaceSidebar.vue`

**Step 1：`main.rs` setup 里加探测（失败时只记录警告，不阻塞）**

```rust
tauri::async_runtime::spawn(async {
    if let Err(e) = crate::git::runner::run(".", &["--version"], 5).await {
        tracing::warn!(?e, "git 未安装或不可用");
    }
});
```

**Step 2：`WorkspaceSidebar.vue` 加路径校验徽章**

```ts
import { onMounted } from 'vue';
import { gitApi } from '../api/git';
const validMap = ref<Record<string, boolean>>({});
async function validateAll() {
  for (const w of store.list) {
    try { const r = await gitApi.validateRepo(w.path); validMap.value[w.id] = r.valid; }
    catch { validMap.value[w.id] = false; }
  }
}
watch(() => store.list.length, validateAll, { immediate: true });
```

模板里在名字后加：

```vue
<n-tag v-if="validMap[w.id] === false" size="tiny" type="error">失效</n-tag>
```

**Step 3：提交**

```bash
git add -A && git commit -m "feat(p9): 启动探测 git + 失效工作区徽章"
```

### Task 9.2：窗口尺寸记忆 + 日志初始化

**Files:**
- Modify: `src-tauri/src/main.rs`、`src-tauri/tauri.conf.json`

**Step 1：tauri 窗口记住大小**

`tauri.conf.json` 的 `app.windows[0]`：

```json
{ "title": "git-compare-tool", "width": 1280, "height": 800, "resizable": true, "minWidth": 960, "minHeight": 600 }
```

**Step 2：`main.rs` 初始化 tracing**

```rust
let _ = tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("info".parse().unwrap()))
    .try_init();
```

（放到 `tauri::Builder::default()` 之前。）

**Step 3：提交**

```bash
git add -A && git commit -m "chore(p9): 窗口尺寸 + tracing 初始化"
```

### Task 9.3：README

**Files:**
- Create: `git-compare-tool/README.md`

仅一页：项目简介 + 启动命令 + 目录结构。

```bash
git add -A && git commit -m "docs: 项目 README"
```

---

## 完成后验收清单

- [ ] `pnpm tauri dev` 启动成功，窗口标题 `git-compare-tool`
- [ ] 新建工作区：路径选择 → 校验 git → 落到 config.json
- [ ] 编辑/删除工作区正常
- [ ] 详情页：分支选择器本地/远程分组，自动载入当前分支
- [ ] 选两个分支自动出 diff；Tree/List 切换正常；+/- 数字与 A/M/D/R 标记正确
- [ ] 点文件右侧显示 unified diff + 语法高亮源文件
- [ ] 底部显示 base..target 提交图
- [ ] 菜单、设置页（主题切换、默认视图）生效并持久化
- [ ] 失效仓库有"失效"徽章，不影响其他仓库
- [ ] 二进制文件/超大 diff 有友好提示
- [ ] 关闭重启，配置保留

---

## 执行方式选择

Plan complete and saved to `git-compare-tool/docs/plans/2026-05-12-git-compare-tool-plan.md`.

两种执行方式：

1. **Subagent-Driven（当前会话）** — 每个 task 派一个 subagent 独立完成，任务间审阅，快速迭代。
2. **Parallel Session（新会话）** — 另起 session，用 `executing-plans` 技能按批次跑。

请选一个。

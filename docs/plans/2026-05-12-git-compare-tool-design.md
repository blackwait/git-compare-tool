# git-compare-tool 设计文档

- 日期：2026-05-12
- 作者：Kiro（经用户逐节确认）
- 目标：基于 Tauri 的桌面端 Git 管理 / 分支比对工具

## 1. 背景与目标

面向开发者的桌面端 Git 仓库浏览与分支比对工具。核心能力：

- 多工作区（每个工作区 = 一个本地 Git 仓库配置项），支持增删改查
- 选择本地 / 远程分支作为"原始分支"和"目标分支"，进行比对
- 文件变更支持 **Tree 视图** 与 **List 平铺视图** 双切换
- 标记文件状态：新增 / 修改 / 删除 / 重命名
- 只读代码查看器，单文件可查看源码与 unified diff
- 提交历史图形化展示 base..target 的提交链
- 顶部菜单集成管理入口

YAGNI：第一版不做提交、push、pull、merge、冲突解决、stash、tag，只做浏览与比对。

## 2. 技术栈

| 层 | 选型 |
|---|---|
| 桌面框架 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite |
| 组件库 | Naive UI |
| 状态管理 | Pinia |
| Diff 渲染 | diff2html 或自定义 + highlight.js |
| 后端语言 | Rust |
| Git 调用 | 直接 spawn 系统 `git` 命令（不使用 libgit2） |
| 配置存储 | JSON 文件（`<appData>/git-compare-tool/config.json`） |
| 日志 | `tracing` + `tracing-subscriber`，文件轮转 |

## 3. 架构

单体前端 + 薄后端：

```
┌───────────────────────────────┐     Tauri IPC     ┌───────────────┐
│ Frontend (Vue 3 + TS)         │ ── invoke ──────▶ │ Rust Backend  │
│  - views / components         │ ◀── result ────── │  commands/    │
│  - stores (Pinia)             │                   │  git/ config/ │
│  - api/ (invoke 封装)          │                   └──────┬────────┘
└───────────────────────────────┘                          │
                                                    spawn `git` ──▶ 本地仓库
                                                    read/write ──▶ config.json
```

前端组件 → `api/` 调 `invoke()` → Rust command → `git/` 模块 spawn `git xxx`、解析输出 → 结构化 JSON 返回。

## 4. 数据模型

### 4.1 配置文件结构

位置：`<appDataDir>/git-compare-tool/config.json`

```json
{
  "version": 1,
  "workspaces": [
    {
      "id": "ws_01h9...",
      "name": "job-manage",
      "path": "D:/code/job/job-manage",
      "remoteName": "origin",
      "createdAt": 1715500000000,
      "updatedAt": 1715500000000
    }
  ],
  "settings": {
    "theme": "dark",
    "defaultView": "tree"
  }
}
```

### 4.2 运行时类型（TypeScript）

```ts
interface Workspace {
  id: string; name: string; path: string;
  remoteName?: string;
  createdAt: number; updatedAt: number;
}

interface Branch {
  name: string;                          // "main" 或 "origin/feature/x"
  kind: 'local' | 'remote';
  isHead: boolean;
  upstream?: string;
}

type ChangeKind = 'added' | 'modified' | 'deleted' | 'renamed';

interface FileChange {
  path: string; oldPath?: string;
  kind: ChangeKind;
  additions: number; deletions: number;
}

interface BranchDiff {
  baseRef: string; targetRef: string;
  files: FileChange[];
  totalAdditions: number; totalDeletions: number;
  truncated?: boolean;
}

interface FileDiff {
  path: string; oldPath?: string;
  isBinary?: boolean;
  hunks: DiffHunk[];
  truncated?: boolean;
}

interface DiffHunk {
  oldStart: number; oldLines: number;
  newStart: number; newLines: number;
  lines: DiffLine[];
}

interface DiffLine {
  kind: 'context' | 'add' | 'del';
  content: string;
  oldLineNo?: number; newLineNo?: number;
}

interface Commit {
  hash: string; shortHash: string;
  parents: string[];
  author: string; date: number;
  message: string;
}
```

Rust 侧对应 struct 用 `serde` + `#[serde(rename_all = "camelCase")]`。

## 5. IPC 命令清单

### 配置管理

| 命令 | 入参 | 返回 |
|---|---|---|
| `config_load` | — | `Config` |
| `workspace_list` | — | `Workspace[]` |
| `workspace_create` | `{name, path, remoteName?}` | `Workspace` |
| `workspace_update` | `{id, patch}` | `Workspace` |
| `workspace_delete` | `{id}` | `void` |
| `workspace_pick_dir` | — | `string \| null` |

### Git 查询（全部只读）

| 命令 | 入参 | 对应 git 命令 |
|---|---|---|
| `git_validate_repo` | `{path}` | `git rev-parse --show-toplevel` |
| `git_list_branches` | `{repoId}` | `git branch -a --format=...` |
| `git_fetch` | `{repoId, remote?}` | `git fetch --prune` |
| `git_current_branch` | `{repoId}` | `git symbolic-ref --short HEAD` |
| `git_diff_branches` | `{repoId, base, target}` | `git diff --numstat` + `--name-status -z` |
| `git_file_diff` | `{repoId, base, target, path}` | `git diff base..target -- <path>` |
| `git_file_content` | `{repoId, ref, path}` | `git show <ref>:<path>` |
| `git_log_between` | `{repoId, base, target, limit?}` | `git log base..target --format=...` |

约定：

- 前端永远传 `repoId`（= workspace.id），由后端映射到绝对路径
- 每个 git 命令 `cwd = workspace.path`，`--no-pager`，显式 `--format`
- 所有 git 输出走 `-z` + NUL 分隔，避免文件名含特殊字符时解析错乱
- 命令默认超时 30s；`git_fetch` 放宽到 120s
- 路径统一 normalize 为 `/`（Windows `\` 转 `/`）

## 6. UI 布局与交互

### 6.1 主窗口布局

```
┌────────────────────────────────────────────────────────┐
│ 菜单: 文件  仓库  视图  帮助              [主题切换]   │
├──────────┬─────────────────────────────────────────────┤
│          │ [仓库标题]  [原始▼] vs [目标▼]  [Fetch]    │
│ 仓库列表 │ [Tree|List] [仅变更/全部]                   │
│ (侧栏)   │ ├────────────────┬──────────────────────┐  │
│          │ │  文件变更面板   │  Diff / 源码查看     │  │
│ + 新建   │ │ (Tree 或 List) │  [Diff | 源文件]     │  │
│ - job-A  │ │                │                      │  │
│ - job-B  │ │ ± 图标 + 数字   │  unified diff        │  │
│          │ ├────────────────┴──────────────────────┤  │
│          │ │  提交历史图 (base..target)             │  │
│          │ └────────────────────────────────────────┘  │
└──────────┴─────────────────────────────────────────────┘
```

### 6.2 路由

- `/` — 欢迎页（无工作区时引导新建）
- `/repo/:id` — 仓库比对主界面
- `/settings` — 设置

### 6.3 关键交互

- **新建工作区**：菜单 / 侧栏"+" → Modal（名称 + 路径选择） → `workspace_pick_dir` + `git_validate_repo` → 通过才入库
- **编辑 / 删除**：侧栏条目右键或悬浮按钮
- **切分支**：两个下拉选择器，来源 `git_list_branches`，本地 / 远程分组 + 过滤
- **Tree/List 切换**：顶部按钮组，持久化到 `settings.defaultView`
- **点击文件**：右侧 `DiffViewer` 懒加载 `git_file_diff`
- **Diff 视图**：unified 格式，行号 + 语法高亮
- **提交图**：SVG 渲染 base..target 链（合并提交分叉可视化）

### 6.4 颜色约定

- 🟢 added / 绿色
- 🟡 modified / 黄色
- 🔴 deleted / 红色
- 🔵 renamed / 蓝色

## 7. 错误处理与边界

### 7.1 统一错误类型

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    ConfigIo(String),
    NotARepo(String),
    WorkspaceNotFound(String),
    GitNotFound,
    GitFailed(String),
    Timeout,
    InvalidArg(String),
    Io(String),
}
```

前端全局 try/catch，Naive UI `useMessage` / `useNotification` 统一提示。

### 7.2 边界场景

| 场景 | 处理 |
|---|---|
| 选的目录不是 git 仓库 | `NotARepo` → Modal 红字提示，不落盘 |
| 工作区路径失效 | 打开时再校验，标失效徽章，允许编辑或删除 |
| 系统无 git | 启动探测 `git --version`，失败弹窗引导安装 |
| fetch 失败 | 弹 toast，不影响本地比对 |
| base = target | files=[]，前端 Empty "没有差异" |
| base/target 不存在 | `GitFailed` → Reset 选择器 |
| 二进制文件 | `isBinary=true`, hunks=[]，前端提示不支持预览 |
| 超大 diff（>5MB / >10000 行） | 后端截断，`truncated=true`，前端提示 |
| 文件名含特殊字符 / 中文 | 用 `-z` + NUL 解析 |
| Windows 路径 | Rust 侧 normalize 为 `/` |
| 重命名 + 修改 | `kind=renamed` + `oldPath` |
| 快速切换分支 | 前端 requestId + AbortController，只渲染最新 |
| 配置损坏 | 备份 `.bak` + 新建空配置 |

### 7.3 日志

`tracing` 写入 `<appData>/git-compare-tool/logs/app.log`，按日期轮转。

## 8. 项目目录结构

```
git-compare-tool/
├── package.json, vite.config.ts, tsconfig.json, index.html
├── src/
│   ├── main.ts, App.vue, router/index.ts
│   ├── stores/ (workspaces.ts, repo.ts, diff.ts)
│   ├── api/ (config.ts, git.ts)
│   ├── types/index.ts
│   ├── views/ (Welcome.vue, RepoDetail.vue, Settings.vue)
│   ├── components/
│   │   (AppMenu, WorkspaceSidebar, WorkspaceEditModal,
│   │    BranchPicker, FileChangeTree, FileChangeList,
│   │    DiffViewer, CommitGraph, EmptyHint)
│   └── assets/
├── src-tauri/
│   ├── Cargo.toml, tauri.conf.json, build.rs
│   └── src/
│       ├── main.rs, error.rs
│       ├── config/ (mod.rs, store.rs)
│       ├── git/ (mod.rs, runner.rs, branches.rs, diff.rs, log.rs)
│       └── commands/ (mod.rs, workspace.rs, git.rs)
├── docs/plans/2026-05-12-git-compare-tool-design.md
└── README.md
```

## 9. 实施阶段

按依赖顺序、每阶段都能跑起来：

| 阶段 | 目标 |
|---|---|
| P1 | 脚手架：Tauri + Vue + Naive UI，空窗口 |
| P2 | 配置与工作区 CRUD：JSON 读写、侧栏、Modal |
| P3 | Git 基础：校验、分支列表、fetch、分支选择器 |
| P4 | 分支 Diff：文件变更 List 视图 + ± 标记 |
| P5 | Tree 视图 + 视图切换 |
| P6 | 文件 Diff：unified 展示、语法高亮、源文件查看 |
| P7 | 提交图：base..target 的 SVG 链 |
| P8 | 菜单 + 设置页（主题、默认视图） |
| P9 | 打磨：错误提示、快捷键、窗口尺寸记忆 |

## 10. 验收

每阶段结束：

- `cargo check` 通过
- `vue-tsc --noEmit` 通过
- `pnpm tauri dev` 能启动并手工走一遍该阶段功能

不编写自动化测试（遵循仓库规则）。

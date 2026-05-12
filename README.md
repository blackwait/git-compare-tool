# git-compare-tool

Tauri 2 + Vue 3 + TypeScript + Naive UI 的桌面端 Git 分支比对工具。

## 功能

- 多工作区：一个工作区对应一个本地 Git 仓库配置项，支持增删改查
- 分支选择：本地 / 远程分组，支持过滤
- 分支比对：选两个分支即可展示文件变更
- 双视图：Tree（按目录层级）与 List（平铺）可切换
- 变更标记：A / M / D / R（新增 / 修改 / 删除 / 重命名），带 +/- 行数
- 只读 Diff 查看：unified 格式，按文件点击懒加载
- 源文件查看：目标分支下的完整源码，语法高亮（highlight.js）
- 提交图：展示 `base..target` 的 commit 链
- 设置：主题（深色 / 浅色 / 跟随系统）、默认视图（Tree / List）
- 顶部菜单：新建工作区、刷新、跳转设置

## 目录结构

```
git-compare-tool/
├── src/                   # Vue 前端
│   ├── api/               # invoke 封装
│   ├── components/        # Sidebar/Modal/BranchPicker/FileChange*/DiffViewer/CommitGraph/AppMenu
│   ├── stores/            # Pinia
│   ├── types/             # TS 类型定义
│   ├── views/             # Welcome / RepoDetail / Settings
│   ├── router/
│   ├── App.vue
│   └── main.ts
├── src-tauri/             # Rust 后端
│   └── src/
│       ├── commands/      # Tauri 命令入口
│       ├── config/        # JSON 配置读写
│       ├── git/           # git CLI 包装 + 解析
│       ├── error.rs
│       ├── lib.rs
│       └── main.rs
├── docs/plans/            # 设计与实施计划
└── .cargo/config.toml     # Cargo 镜像（rsproxy.cn）
```

## 启动

系统要求：Node.js 18+、Rust 1.75+（MSVC 工具链，Windows 需装 Visual Studio Build Tools + C++ 桌面开发工作负载）、系统 `git`。

```bash
pnpm install
pnpm tauri dev
```

## 配置文件位置

- Windows: `%APPDATA%\com.kiro.git-compare-tool\git-compare-tool\config.json`
- macOS: `~/Library/Application Support/com.kiro.git-compare-tool/git-compare-tool/config.json`

## 技术说明

- 后端通过 spawn 系统 `git` 命令执行查询，30s 超时（fetch 120s）
- `git diff --name-status -z` + `git diff --numstat -z` 合并构造变更列表，`-z` 避免文件名特殊字符问题
- `git diff -U3` 输出交给 Rust 侧简易 unified parser 解析为结构化 hunks
- 所有路径统一 normalize 为 `/`

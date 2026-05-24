# AGENTS.md

## 项目概览

workc 是一个 Rust workspace（4 crate，edition 2024，MSRV 1.88），本地工作流 CLI。

```
crates/
  workc-domain/        # 聚合、实体、值对象、仓库 trait（无外部依赖）
  workc-application/   # 用例、DTO、port 抽象（依赖 domain）
  workc-infrastructure/# FS 仓储、Git 客户端、编辑器启动、时钟（依赖 domain+application）
  workc-cli/           # Clap CLI、presenter（text/json）、组合根
scripts/
  smoke_test.py        # Python 3 冒烟测试套件
  install.py            # 一行安装脚本（在线下载二进制 + 初始化）
  init.py               # 全局数据初始化（幂等）
Makefile.toml          # cargo-make 任务定义
```

## 安装（面向用户）

```
# 在线一行安装
python3 -c "$(curl -fsSL https://raw.githubusercontent.com/org/workc/main/scripts/install.py)"

# 离线（从 Release zip 解压后）
python3 install.py
```

脚本从 GitHub Releases 下载预编译二进制，安装到 `~/.workc/bin/`，不依赖 Rust 工具链。

卸载：`python3 -c "$(curl -fsSL https://raw.githubusercontent.com/org/workc/main/scripts/uninstall.py)"`

## 发布新版本

推送 `v*` tag 触发 GitHub Actions，自动构建四个平台的 release 产物：

```
git tag v0.1.0
git push origin v0.1.0
```

每个 zip 包含：`workc` 二进制 + `install.py` + `init.py` + `uninstall.py`。

## 数据模型

| 位置 | 内容 | 说明 |
|---|---|---|
| `~/.workc/` | 全局配置目录 | 跨 workspace 共享 |
| `<项目>/.workc.toml` | workspace 元数据 | 一个目录一个 task |

### `~/.workc/` — 全局数据

```
~/.workc/
├── bin/workc                         # 二进制（安装时放入）
├── config.toml                        # [knowledge] remote
├── workspaces.toml                    # [[workspaces]] 注册表
├── repos/catalog.toml                 # [[repos]] 仓库注册表
├── repos/groups.toml                  # [[groups]] 仓库分组
├── skills/registry/{sources,skills}.toml  # 技能注册表
├── skills/cache/<id>/<ver>/           # 技能文件缓存
└── knowledge/                         # git clone（已发布知识）
```

### `<项目>/` — workspace

```
<项目>/
├── .workc.toml                        # task 元数据
├── repos/                             # 克隆的仓库
├── knowledge-candidates/<cid>/        # 知识草稿
└── skills/mounts.toml                 # [{skill_id, version}] 挂载引用
```

## CLI 命令全集

| 命令 | 子命令 | 说明 |
|---|---|---|
| `list` | — | 列出任务，支持 `--status` / `--tag` / `--limit`，`--json` |
| `open` | — | 在编辑器中打开任务，需 `--editor cursor\|vscode` |
| `task create` | — | 创建任务，支持 `--slug`、`--title`、`--template`、`--tags`、`--repos`、`--repo-groups`、`--skills` |
| `task close` | — | 关闭任务 |
| `task repos` | `set/add/remove/clone/status` | 任务仓库管理 |
| `repo` | `add/list` | 仓库目录 |
| `repo-group` | `add/list` | 仓库分组 |
| `skill import` | — | 导入技能（git/local/archive），`--skill-id` 缺省时自动从 `--name` 派生 |
| `skill show/versions` | — | 查看技能定义、版本列表 |
| `skill mount/mounts/unmount` | — | 任务技能挂载管理 |
| `skill check-updates` | — | 检查技能版本更新 |
| `knowledge candidate` | `create/list/show/update-meta/delete` | 知识候选 CRUD |
| `knowledge` | `list/show/update-meta/delete/promote` | 全局知识管理 |

## 前提条件

| 操作 | 前提 |
|---|---|
| `open <task>` | 必须传 `--editor cursor` 或 `--editor vscode` |
| `task create --skills <id>` | 技能需预先通过 `skill import` 导入，否则 mount 阶段报 "skill not found" |
| `task repos clone` | 需先通过 `task repos set` 关联仓库，且 `git` 在 PATH |
| `skill mount` | 技能需已有版本（`skill import` 时传 `--version`），否则报 "skill version is required" |
| `skill mounts` | `<TASK>` 必须传 `task-*` 格式的完整 task ID |

## 开发者

### 日常流程

```powershell
# 改代码后，快速检查编译
cargo check --workspace

# 格式 + lint
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings

# 单测
cargo test

# 冒烟测试（自动构建 release）
cargo make smoke
```

### 提交前

```powershell
cargo make ci            # fmt → lint → smoke 一键通过后再 commit
```

### 发布

```powershell
git tag v0.1.0
git push origin v0.1.0   # 触发 GitHub Actions，构建 4 平台 release
```

### 构建 & 运行

```powershell
cargo build --release -p workc-cli        # 产物：target/release/workc-cli.exe
cargo run -p workc-cli -- <args>          # Debug 运行
```

### 测试

- 单测：`cargo test`（16 个）
- 冒烟测试：`cargo make smoke` / `cargo make smoke-cli` / `cargo make smoke-infra`
- 直接跑冒烟（跳过构建）：`python scripts/smoke_test.py`
- 当前冒烟 45 项全部通过

### Windows 注意事项

- Shell：PowerShell 7+（`pwsh`）
- Makefile.toml 路径用 camelCase；`cargo make` 参数中的反斜杠可能需要转义
- `WindowsEditorLauncher` 使用 `cmd /c start`——`open` 会弹出新窗口

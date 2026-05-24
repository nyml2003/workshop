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
Makefile.toml          # cargo-make 任务定义
```

## 核心约定：CWD 即 workspace 根目录

**所有数据持久化在当前工作目录下。**CLI 以 `std::env::current_dir()` 作为 workspace 根目录。在不同目录下运行会生成独立的 workspace，没有全局配置目录。

## 构建 & 运行

```powershell
cargo build --release -p workc-cli        # 产物：target/release/workc-cli.exe
cargo run -p workc-cli -- <args>          # Debug 运行
cargo check --workspace                    # 快速编译检查
cargo fmt --all -- --check                # 格式检查
cargo clippy --all-targets -- -D warnings # Lint
```

## 测试

- 应用层 11 个单测 + 基础设施层 5 个单测，全部通过。
- **冒烟测试** 使用 `cargo make`（需 `cargo install cargo-make`）和 Python 3。
  - `cargo make smoke` — 完整 CLI + infra 冒烟测试（自动构建 release）
  - `cargo make smoke-cli` — 仅 CLI 命令
  - `cargo make smoke-infra` — 仅文件系统/TOML 校验
  - `python scripts/smoke_test.py` — 直接运行（跳过构建；macOS/Linux 用 `python3`）
- 冒烟测试创建临时 workspace，对所有 CLI 命令执行校验（退出码、输出内容、磁盘产物），最后清理。当前 47 项全部通过。

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

## 持久化布局

```
<CWD>/
  tasks/<task-id>/task.toml                       # 任务元数据
  tasks/<task-id>/repos/                           # 已克隆仓库
  tasks/<task-id>/knowledge-candidates/<cid>/      # 知识草稿
  tasks/<task-id>/.codex/skills/mounts.toml        # 技能挂载
  repos/catalog.toml                               # [[repos]]
  repos/groups.toml                                # [[groups]]
  skills/registry/sources.toml                     # 技能来源
  skills/registry/skills.toml                      # 技能定义
  knowledge/<kid>/meta.toml                        # 已发布知识
```

## Windows 注意事项

- Shell：PowerShell 7+（`pwsh`），不要用 `powershell.exe`。
- Makefile.toml 路径用 camelCase；`cargo make` 参数中的反斜杠可能需要转义。
- `WindowsEditorLauncher` 使用 `cmd /c start`——如果编辑器已安装，`open` 会弹出新窗口。

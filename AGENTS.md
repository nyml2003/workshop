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

**所有数据持久化在当前工作目录下。**CLI 以 `std::env::current_dir()` 作为 workspace 根目录。在不同目录下运行会生成独立的 workspace——这是刻意设计，没有全局配置目录。

## 构建 & 运行

```powershell
cargo build --release -p workc-cli        # 产物：target/release/workc-cli.exe
cargo run -p workc-cli -- <args>          # Debug 运行
cargo check --workspace                    # 快速编译检查
cargo fmt --all -- --check                # 格式检查
cargo clippy --all-targets -- -D warnings # Lint
```

## 测试

- **没有单测。** 项目处于骨架/MVP 阶段。
- **冒烟测试** 使用 `cargo make`（需安装 cargo-make：`cargo install cargo-make`）。同时需要 Python 3。
  - `cargo make smoke` — 完整 CLI + infra 冒烟测试（自动构建 release）
  - `cargo make smoke-cli` — 仅 CLI 命令
  - `cargo make smoke-infra` — 仅文件系统/TOML 校验
  - `python scripts/smoke_test.py` — 直接运行（跳过构建；macOS/Linux 用 `python3`）
- 冒烟测试会创建临时 workspace，对 release 产物执行所有 CLI 命令，校验退出码和输出内容，检查磁盘产物，最后清理。

## 已知缺口（命令会拒绝或不可用）

| 触发条件 | 行为 |
|---|---|
| `task create --skills ...` | 服务层不再拦截，但 CLI 在任务创建后自动挂载技能——若技能未预先导入，mount 报 "skill not found" |
| `open <task>` 不带 `--editor` | 失败——必须传 `--editor cursor` 或 `--editor vscode` |
| `skill prepare` / `skill use` | 返回 `AdapterUnavailable`（runtime 为占位实现） |
| `task repos clone` | 需要 `git` 在 PATH 且仓库可达 |
| JSON 输出（`--json`） | JSON presenter 是桩，V1 未实现 |

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

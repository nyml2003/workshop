# 架构说明

## 分层总览

workc 采用四层架构，调用方向自上而下，依赖方向自下而上（`→` = 调用/依赖）：

```
┌──────────────────────────────────────┐
│            workc-cli                 │  命令解析 · 输出渲染 · 组合根
├──────────────────────────────────────┤
│         workc-application            │  用例编排 · DTO · Ports 抽象
├──────────────────────────────────────┤
│           workc-domain               │  聚合 · 实体 · 值对象 · Repository 特征
├──────────────────────────────────────┤
│        workc-infrastructure          │  FS 仓储 · Git · Editor · Clock · Runtime
└──────────────────────────────────────┘
```

- **workc-domain** 无外部依赖，是最内层。
- **workc-application** 依赖 domain，定义 ports（特征接口）让基础设施实现。
- **workc-infrastructure** 依赖 domain + application，提供 ports 的具体实现。
- **workc-cli** 在最外层，负责组装所有依赖并启动。

## 数据分层

workc 的数据分为两层：

| 层级 | 位置 | 内容 |
|---|---|---|
| 全局 | `~/.workc/` | 仓库目录、技能注册表与缓存、已发布知识、workspace 注册表 |
| 项目 | `<项目>/` | task 元数据（`.workc.toml`）、克隆的仓库、知识草稿、技能挂载引用 |

全局数据跨 workspace 共享，项目数据跟随 task 目录隔离。一个目录为一个 task。

## 模块职责

### 1. 任务 (task)

**核心聚合：** `TaskWorkspace`（含 `TaskMeta`、`TaskRepoSelection`、`TaskActivity`、`TaskPaths`）

- 创建任务时生成唯一 ID（格式 `task-YYYYMMDD-<slug-hint>`），写入 `.workc.toml` 到当前目录。
- 状态生命周期：`Draft → Active → Closed → Archived`。
- `mark_opened` 记录最后打开时间和编辑器。
- `close` 将状态置为 Closed。
- Workspace 创建后自动注册到 `~/.workc/workspaces.toml`，供 `list` / `open` 查找。

### 2. 仓库目录 (repo_catalog)

**核心聚合：** `RepoCatalog`（含 `RepoEntry` 列表和 `RepoGroup` 列表），存储于 `~/.workc/repos/`。

- 维护一个全局仓库注册表和分组。
- `RepoGroup` 将多个 `RepoId` 组合为一个逻辑组，供任务批量引用。

### 3. 任务仓库 (task_repos)

**应用服务：** `TaskReposApplicationService`

- 将全局仓库目录中的仓库关联到具体任务，克隆到任务目录下的 `repos/`。
- 提供 `clone` 和 `status` 命令，支持 `--missing`、`--dry-run` 等过滤。
- `GitClient` 为可选 port，未注入时克隆和状态操作返回 `AdapterUnavailable`。

### 4. 技能注册 (skill_registry)

**核心聚合：** `SkillRegistry`（含 `SkillSource` 和 `SkillDefinition`），存储于 `~/.workc/skills/registry/`。

- `import` 从外部源（git / local / archive）导入技能定义，文件缓存到 `~/.workc/skills/cache/<id>/<version>/`。
- `SkillMountPlanner` 作为 domain service 规划挂载版本。

### 5. 任务技能 (task_skills)

**应用服务：** `TaskSkillsApplicationService`

- `mount` 将全局技能注册表中的技能挂载到任务，在任务的 `skills/mounts.toml` 中记录 `{skill_id, version}` 引用。
- `prepare` / `use` / `check_prepare_status` 均依赖 `SkillRuntime`，当前 runtime 为 placeholder。
- `override_skill` 未实现，返回 `AdapterUnavailable`。

### 6. 知识 (knowledge)

**核心实体：** `KnowledgeCandidate`（任务作用域草稿）、`KnowledgeEntry`（全局发布）

- 候选在任务目录的 `knowledge-candidates/<candidate-id>/` 下创建、编辑、删除。
- `promote` 将候选提升为全局知识条目，写入 `~/.workc/knowledge/<kid>/`，同时删除原候选。
- `~/.workc/knowledge/` 是一个 git clone（remote 来自 `~/.workc/config.toml`），promote 后由用户手动 git 操作。

### 7. 全局配置 (config)

**新增：** `WorkcConfig` 聚合，存储于 `~/.workc/config.toml`。

- 当前包含 `[knowledge].remote`，为知识 git 仓库的远程地址。
- 后续可扩展 `[skills]`、`[editor]` 等 section。

### 8. Workspace 注册表 (workspace)

**新增：** `WorkspaceEntry` 实体，存储于 `~/.workc/workspaces.toml`。

- 记录所有 workspace 的 slug、路径、标题、状态、最后活动时间。
- `task create` 时自动注册，`task close` 时更新状态。

## Ports 抽象

应用层通过以下特征接口与外部世界交互，所有实现在 infrastructure 层：

| Port | 职责 | 实现 |
|---|---|---|
| `Clock` | 获取当前时间 | `SystemClock`（`OffsetDateTime::now_utc()`） |
| `GitClient` | 克隆、状态、拉取 | `CommandGitClient`（调 `git` CLI） |
| `EditorLauncher` | 在编辑器中打开目录 | `MacOsEditorLauncher` / `WindowsEditorLauncher` |
| `SkillRuntime` | 执行技能 prepare/use/status | macOS / Windows 均为 placeholder |
| `ConfigRepository` | 全局配置读写 | `FsConfigRepository` |
| `WorkspaceRegistryRepository` | workspace 注册表读写 | `FsWorkspaceRegistryRepository` |

`GitClient` 和 `SkillRuntime` 在应用服务中均为可选注入——当 CLI 未提供实现时，相关操作返回 `AdapterUnavailable`。

## 核心数据流

### 创建任务

```
CLI: task create --slug x --title y --template z
  → Application: create_task(CreateTaskCommand)
    → TaskIdGenerator 生成 ID
    → TaskWorkspace::create() 构造聚合
    → TaskRepository::save()
      → Infrastructure: FsTaskRepository 创建目录树（repos/、materials/、knowledge-candidates/、skills/）
                       并序列化 .workc.toml 到当前目录
    → WorkspaceRegistryRepository 注册 workspace
  ← CreateTaskResult { task_id, slug, title, template }
```

### 打开任务

```
CLI: open <task-ref> --editor vscode
  → Application: open_task(OpenTaskCommand)
    → 检查 editor 非空（必须显式传入）
    → WorkspaceRegistryRepository 查找 workspace 路径
    → EditorLauncher::open_dir(task_root_path, EditorKind::VsCode)
    → task.mark_opened(now, editor)
    → TaskRepository::save()
  ← ()
```

### 知识候选提升为全局知识

```
CLI: knowledge candidate create <task-id> <candidate-id> --title "..." --category "..."
  → Application: create_candidate(CreateKnowledgeCandidateCommand)
    → Infrastructure: FsKnowledgeRepository::create_candidate()
      写入 <CWD>/knowledge-candidates/<candidate-id>/meta.toml
  ← CandidateMutationResult

CLI: knowledge promote <task-id> <candidate-id> <knowledge-id>
  → Application: promote(PromoteKnowledgeCommand)
    → Infrastructure: FsKnowledgeRepository::promote_candidate()
      将候选内容复制到 ~/.workc/knowledge/<knowledge-id>/meta.toml
      删除原候选
  ← PromoteKnowledgeResult
```

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

## 模块职责

### 1. 任务 (task)

**核心聚合：** `TaskWorkspace`（含 `TaskMeta`、`TaskRepoSelection`、`TaskActivity`、`TaskPaths`）

- 创建任务时生成唯一 ID（格式 `task-YYYYMMDD-<slug-hint>`），落地目录和 `task.toml`。
- 状态生命周期：`Draft → Active → Closed → Archived`。
- `mark_opened` 记录最后打开时间和编辑器。
- `close` 将状态置为 Closed。

### 2. 仓库目录 (repo_catalog)

**核心聚合：** `RepoCatalog`（含 `RepoEntry` 列表和 `RepoGroup` 列表）

- 维护一个全局仓库注册表和分组。
- `RepoGroup` 将多个 `RepoId` 组合为一个逻辑组，供任务批量引用。
- 当前不执行仓库选择或验证的额外逻辑（domain services 预留为空）。

### 3. 任务仓库 (task_repos)

**应用服务：** `TaskReposApplicationService`

- 将仓库目录中的仓库（直接指定或通过 repo-group 解析）关联到具体任务。
- 提供 `clone` 和 `status` 命令，支持 `--missing`、`--dry-run` 等过滤。
- `GitClient` 为可选 port，未注入时克隆和状态操作返回 `AdapterUnavailable`。

### 4. 技能注册 (skill_registry)

**核心聚合：** `SkillRegistry`（含 `SkillSource` 和 `SkillDefinition`）

- `import` 从外部源（git / local / archive）导入技能定义。
- 每个技能可包含多个 `PrepareStep` 和 `UseStep`，定义生命周期动作。
- `SkillMountPlanner` 作为 domain service 规划挂载版本。

### 5. 任务技能 (task_skills)

**应用服务：** `TaskSkillsApplicationService`

- `mount` 将技能注册表中的技能挂载到任务，产生 `TaskSkillMount`。
- 挂载点路径：`tasks/<task-id>/.codex/skills/mounted/<mount-id>/`。
- `prepare` / `use` / `check_prepare_status` 均依赖 `SkillRuntime`，当前 runtime 为 placeholder。
- `override_skill` 未实现，返回 `AdapterUnavailable`。

### 6. 知识 (knowledge)

**核心实体：** `KnowledgeCandidate`（任务作用域草稿）、`KnowledgeEntry`（全局发布）

- 候选在任务下创建、编辑、删除。
- `promote` 将候选提升为全局知识条目，同时可覆盖元信息。
- 候选路径：`tasks/<task-id>/knowledge-candidates/<candidate-id>/`，全局知识路径：`knowledge/<knowledge-id>/`。

## Ports 抽象

应用层通过以下特征接口与外部世界交互，所有实现在 infrastructure 层：

| Port | 职责 | 实现 |
|---|---|---|
| `Clock` | 获取当前时间 | `SystemClock`（`OffsetDateTime::now_utc()`） |
| `GitClient` | 克隆、状态、拉取 | `CommandGitClient`（调 `git` CLI） |
| `EditorLauncher` | 在编辑器中打开目录 | `MacOsEditorLauncher` / `WindowsEditorLauncher` |
| `SkillRuntime` | 执行技能 prepare/use/status | macOS / Windows 均为 placeholder |

`GitClient` 和 `SkillRuntime` 在应用服务中均为可选注入——当 CLI 未提供实现时，相关操作返回 `AdapterUnavailable`。

## 核心数据流

### 创建并打开任务

```
CLI: task create --slug x --title y --template z
  → Application: create_task(CreateTaskCommand)
    → 检查 phase-gate：拒绝 repo-groups 和 initial-skills
    → TaskIdGenerator 生成 ID
    → TaskWorkspace::create() 构造聚合
    → TaskRepository::save()
      → Infrastructure: FsTaskRepository 创建目录树并序列化 task.toml
  ← CreateTaskResult { task_id, slug, title, template }

CLI: open <task-ref> --editor vscode
  → Application: open_task(OpenTaskCommand)
    → 检查 editor 非空（当前阶段必须显式传入）
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
      写入 tasks/<task-id>/knowledge-candidates/<candidate-id>/meta.toml
  ← CandidateMutationResult

CLI: knowledge promote <task-id> <candidate-id> <knowledge-id>
  → Application: promote(PromoteKnowledgeCommand)
    → Infrastructure: FsKnowledgeRepository::promote_candidate()
      将候选内容复制到 knowledge/<knowledge-id>/meta.toml
      同时保留原候选
  ← PromoteKnowledgeResult
```

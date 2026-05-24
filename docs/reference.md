# 参考手册

## CLI 命令

所有命令通过 `cargo run -p workc-cli -- <command>` 执行。

### 顶层命令

| 命令 | 子命令 | 说明 |
|---|---|---|
| `list` | — | 列出任务，支持 `--status` / `--tag` / `--limit` 过滤 |
| `open <TASK>` | — | 在编辑器中打开任务，需 `--editor cursor\|vscode` |
| `task` | `create` / `close` / `repos` | 任务管理 |
| `repo` | `add` / `list` | 仓库目录 |
| `repo-group` | `add` / `list` | 仓库分组 |
| `skill` | `import` / `show` / `versions` / `mount` / `mounts` / `unmount` | 技能管理 |
| `knowledge` | `candidate` / `list` / `show` / `update-meta` / `delete` / `promote` | 知识管理 |

### task 命令组

```
task create   --slug <SLUG> --title <TITLE> --template <TMPL>
              [--description <DESC>] [--source-brief <BRIEF>] [--tags <TAGS>...]
              [--repo-groups <GROUPS>...] [--repos <REPOS>...] [--skills <SKILLS>...]

task close    <TASK_ID>

task repos set     <TASK> [--repo-groups <GROUPS>...] [--repos <REPOS>...]
task repos add     <TASK> <REPOS>...
task repos remove  <TASK> <REPOS>...
task repos clone   <TASK> [--repo <REPO>]... [--missing] [--dry-run]
task repos status  <TASK> [--repo <REPO>]... [--clone-state missing|ready|dirty|unknown] [--dry-run]
```

注意：`--skills` 在服务层已不再被拦截，但 CLI 会在任务创建后自动尝试挂载指定技能——若技能未预先通过 `skill import` 导入，mount 将失败并报 "skill not found"。

### skill 命令组

```
skill import   <KIND: git|local|archive> <LOCATION> [--name <NAME>] [--version <VER>] [--skill-id <ID>]
skill show     <SKILL_ID>
skill versions <SKILL_ID>
skill mount    <TASK> <SKILL_ID> [VERSION]
skill mounts   <TASK>
skill unmount  <TASK> <MOUNT_ID>
```

注意：`mounts` 的 `<TASK>` 参数当前要求传入 task ID（以 `task-` 开头的完整 ID）。

### knowledge 命令组

```
knowledge candidate create      <TASK_ID> <CANDIDATE_ID> --title <TITLE> [--category <CAT>] [--tags <TAGS>...] [--source <SRCS>...]
knowledge candidate list        <TASK_ID>
knowledge candidate show        <TASK_ID> <CANDIDATE_ID>
knowledge candidate update-meta <TASK_ID> <CANDIDATE_ID> [--title <TITLE>] [--category <CAT>] [--tags <TAGS>...]
knowledge candidate delete      <TASK_ID> <CANDIDATE_ID>

knowledge list
knowledge show        <KNOWLEDGE_ID>
knowledge update-meta <KNOWLEDGE_ID> [--title <TITLE>] [--category <CAT>] [--tags <TAGS>...]
knowledge delete      <KNOWLEDGE_ID>

knowledge promote <TASK_ID> <CANDIDATE_ID> <KNOWLEDGE_ID> [--title <TITLE>] [--category <CAT>] [--tags <TAGS>...]
```

## 持久化布局

workc 将所有数据存储在 workspace 根目录下，格式为 TOML。

```
<workspace_root>/
├── tasks/
│   └── <task-id>/                          # 每个任务一个目录
│       ├── task.toml                        # 任务元信息与活动记录
│       ├── repos/                           # 克隆后的仓库
│       ├── materials/                       # 任务资料
│       └── knowledge-candidates/
│           └── <candidate-id>/
│               ├── meta.toml                # 候选元信息
│               └── sources/
│                   └── source-NNN.toml      # 候选引用来源
│       └── .codex/
│           └── skills/
│               ├── mounts.toml              # 任务技能挂载列表
│               └── mounted/<mount-id>/      # 挂载的技能文件
│
├── repos/
│   ├── catalog.toml                         # [[repos]] 仓库注册表
│   └── groups.toml                          # [[groups]] 仓库分组
│
├── skills/
│   └── registry/
│       ├── sources.toml                     # [[sources]] 技能来源
│       └── skills.toml                      # [[skills]] 技能定义
│
└── knowledge/
    └── <knowledge-id>/
        ├── meta.toml                        # 全局知识条目
        └── sources/
            └── source-NNN.toml              # 知识引用来源
```

### 各文件用途

| 路径 | 用途 |
|---|---|
| `tasks/<task-id>/task.toml` | 任务的核心信息：ID、slug、标题、模板、状态、时间戳、标签、关联仓库和分组、目录路径 |
| `repos/catalog.toml` | 全局仓库注册表：每条记录含 ID、URL、标签、描述 |
| `repos/groups.toml` | 仓库分组：每条记录含组 ID、成员仓库 ID 列表、标签、描述 |
| `skills/registry/sources.toml` | 技能来源：每条记录含来源 ID、类型 (git/local/archive)、位置、版本引用、导入时间 |
| `skills/registry/skills.toml` | 技能定义：每条记录含技能 ID、所属来源、可用版本列表、最新版本 |
| `tasks/<task-id>/.codex/skills/mounts.toml` | 任务技能挂载：每条记录含挂载 ID、技能 ID、版本、挂载时间、状态、路径 |
| `tasks/<task-id>/knowledge-candidates/<candidate-id>/meta.toml` | 候选知识条目：含 ID、标题、分类、标签、时间戳、状态 (candidate) |
| `knowledge/<knowledge-id>/meta.toml` | 全局知识条目：与候选结构相同，状态为 published |

## 已支持与当前限制

### 已支持

- 任务的创建、按状态/标签过滤列表、打开（在编辑器中）和关闭
- 仓库目录的注册和列表
- 仓库分组的创建和列表
- 任务的仓库关联、克隆和状态检查（依赖 git 可用）
- 技能的导入、查看和版本列表
- 技能的挂载、卸载、更新和版本更新检查
- 知识候选的 CRUD（作用域：任务）
- 知识条目的 CRUD（作用域：全局）
- 候选到全局知识的提升
- 文本输出渲染

### 当前限制

| 限制 | 影响 | 代码出处 |
|---|---|---|
| `task create --skills` 创建后自动 mount | 若技能未预先导入，mount 报 "skill not found" | `cli/src/cli/task.rs:231-247` |
| `open` 必须显式传 `--editor` | 不支持编辑器自动发现 | `application/src/task/service.rs:143-146` |
| skill runtime 为 placeholder | `prepare` / `use` / `check_prepare_status` 返回 `AdapterUnavailable` | `infrastructure/src/runtime/macos.rs:3`、`windows.rs:3` |
| `skill override` 未实现 | 调用即返回 `AdapterUnavailable` | `application/src/task_skills/service.rs:146` |
| JSON presenter 不在 V1 范围 | 无 JSON 输出能力 | `cli/src/presenters/json.rs:1` |
| `git ahead/behind` 未计算 | `repo status` 中的 ahead/behind 恒为 0 | `infrastructure/src/git/command_git_client.rs` |
| `skill mounts` 要求 task ID 格式 | 必须传 `task-*` 格式的 ID | CLI help 说明 |

## 术语约定

| 术语 | 含义 |
|---|---|
| workc | 产品名 |
| workc-cli | 当前可运行的二进制 / Cargo package 名 |
| 任务 (task) | 一次开发工作的上下文容器 |
| 仓库目录 (repo catalog) | 全局可用的仓库注册表 |
| 仓库分组 (repo group) | 多个仓库的逻辑组合 |
| 技能 (skill) | 可复用的工具或工作流，含 prepare / use 步骤 |
| 知识候选 (knowledge candidate) | 在任务内产生的知识草稿 |
| 知识条目 (knowledge entry) | 提升后的全局知识 |

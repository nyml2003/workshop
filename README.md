# workc

workc 是一个 Rust workspace 里的本地工作流 CLI，帮助开发者在任务之间快速切换上下文。

## 当前能力

| 能力 | 说明 |
|---|---|
| 任务 (task) | 创建、打开、关闭任务；每个任务拥有独立的目录结构和版本库关联 |
| 仓库目录 (repo / repo-group) | 注册常用仓库及其分组，供任务引用 |
| 任务仓库 (task repos) | 将仓库目录中的仓库关联到任务，支持克隆和状态检查 |
| 技能 (skill) | 从外部源导入技能定义，挂载到任务，管理版本 |
| 知识 (knowledge) | 在任务下创建知识候选，审查后提升为全局知识条目 |

## 最短上手

以下命令均从仓库根目录执行。

### 1. 创建任务

```
cargo run -p workc-cli -- task create \
  --slug my-first-task \
  --title "我的第一个任务" \
  --template default \
  --tags "demo"
```

### 2. 查看任务列表

```
cargo run -p workc-cli -- list
```

### 3. 打开任务

```
cargo run -p workc-cli -- open my-first-task --editor vscode
```

### 4. 注册仓库并关联到任务

```
cargo run -p workc-cli -- repo add api-gateway https://github.com/example/api-gateway
cargo run -p workc-cli -- task repos add <task-id> api-gateway
```

### 5. 导入技能并挂载

```
cargo run -p workc-cli -- skill import git https://github.com/example/my-skill --name my-skill
cargo run -p workc-cli -- skill mount <task-id> my-skill
```

### 6. 记录知识

```
cargo run -p workc-cli -- knowledge candidate create <task-id> candidate-1 \
  --title "认证流程分析" \
  --category "architecture"
cargo run -p workc-cli -- knowledge promote <task-id> candidate-1 knowledge-1
```

## 当前状态

workc 当前处于 **V1 骨架阶段**。以下能力已在 CLI 中暴露参数入口，但后端尚未实现：

| 功能 | 状态 |
|---|---|
| `task create --repo-groups` | 参数接受但服务层拒绝（"repo-group enrichment is not supported in this phase"） |
| `task create --skills` | 参数接受但服务层拒绝（"initial skill mounts are not supported in this phase"） |
| `open <task>` | 必须显式传 `--editor`，不支持默认编辑器自动发现 |
| `skill prepare / use` | skill runtime 在 macOS/Windows 均为 placeholder，暂不可用 |
| JSON 输出 | 不在 V1 范围内 |

所有可用的命令和执行边界详见 [docs/reference.md](docs/reference.md)。

## 继续阅读

| 如果你想了解 | 阅读 |
|---|---|
| 四层架构、模块职责、数据流 | [docs/architecture.md](docs/architecture.md) |
| CLI 命令全集、持久化布局、能力边界 | [docs/reference.md](docs/reference.md) |
| 已完成项、已知缺口、后续方向 | [docs/roadmap.md](docs/roadmap.md) |

# workc

workc 是一个本地工作流 CLI，帮助开发者在任务之间快速切换上下文。

## 安装

```
# 在线一行安装
python3 -c "$(curl -fsSL https://raw.githubusercontent.com/org/workc/main/scripts/install.py)"

# 离线（从 Release zip 解压后）
python3 install.py
```

安装后 `workc` 命令即可用。

卸载：`python3 -c "$(curl -fsSL https://raw.githubusercontent.com/org/workc/main/scripts/uninstall.py)"`

## 当前能力

| 能力 | 说明 |
|---|---|
| 任务 (task) | 创建、打开、关闭任务；每个任务拥有独立的目录结构和版本库关联 |
| 仓库目录 (repo / repo-group) | 注册常用仓库及其分组，全局共享，供任务引用 |
| 任务仓库 (task repos) | 将仓库目录中的仓库关联到任务，支持克隆和状态检查 |
| 技能 (skill) | 从外部源导入技能定义并缓存，挂载到任务，管理版本 |
| 知识 (knowledge) | 在任务下创建知识候选，审查后提升为全局知识条目 |

## 最短上手

### 1. 创建任务

```
workc task create \
  --slug my-first-task \
  --title "我的第一个任务" \
  --template default \
  --tags "demo"
```

### 2. 查看任务列表

```
workc list
```

### 3. 打开任务

```
workc open my-first-task --editor vscode
```

### 4. 注册仓库并关联到任务

```
workc repo add api-gateway https://github.com/example/api-gateway
workc task repos set my-first-task --repos api-gateway
```

### 5. 导入技能并挂载

```
workc skill import git https://github.com/example/my-skill --name my-skill --version 0.1.0
workc skill mount my-first-task my-skill
```

### 6. 记录知识

```
workc knowledge candidate create my-first-task candidate-1 \
  --title "认证流程分析" \
  --category "architecture"
workc knowledge promote my-first-task candidate-1 knowledge-1
```

## 继续阅读

| 如果你想了解 | 阅读 |
|---|---|
| 四层架构、模块职责、数据流 | [docs/architecture.md](docs/architecture.md) |
| CLI 命令全集、持久化布局、能力边界 | [docs/reference.md](docs/reference.md) |
| 已完成项、已知缺口、后续方向 | [docs/roadmap.md](docs/roadmap.md) |

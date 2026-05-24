# 现状与演进

本文档记录 workc 的当前完成度，不承诺任何时间表。

## 已完成

| 功能 | 说明 |
|---|---|
| 任务 CRUD | 创建（含 ID 生成和 slugs 唯一性校验）、列表（按状态/标签过滤、按活动时间排序）、用编辑器打开、关闭 |
| 仓库目录 | `repo add` / `repo list`，`repo-group add` / `repo-group list`，持久化到 `repos/catalog.toml` 和 `repos/groups.toml` |
| 任务仓库关联 | `task repos set/add/remove`，支持通过 repo-group 批量关联；`clone`（含 `--missing`、`--dry-run`）和 `status` 检查 |
| 技能注册 | `skill import` 支持 git、local、archive 三种来源；`show` 和 `versions` 查看；持久化到 `skills/registry/` |
| 技能挂载 | `skill mount/unmount` 将技能关联到任务；版本更新检查（`check_skill_updates`）和升级（`update_skill`） |
| 知识候选 | 在任务下创建、查看、编辑、删除候选；持久化到 `tasks/<task-id>/knowledge-candidates/` |
| 知识条目 | 全局知识的 CRUD；`promote` 将候选提升为全局条目 |
| 四层架构骨架 | 领域层、应用层（含 ports）、基础设施层（FS 仓储、git adapter、editor adapter、system clock）、CLI 层 |
| 跨平台 editor 启动 | macOS（Cursor/VS Code）和 Windows（Cursor/VS Code via `where.exe`） |
| 文本输出 | 所有命令都有对应的文本渲染函数 |
| 测试覆盖 | 11 个应用层测试 + 5 个基础设施层测试，全部通过 |

## 已暴露但未闭环

| 项目 | 现状 |
|---|---|
| `render_skill_updates` 死代码 | 渲染函数已实现但无调用路径，编译时产生 dead_code 警告 |
| `git ahead/behind` 恒为 0 | `get_repo_status` 未从 `git rev-list` 计算真实的 ahead/behind |
| repo catalog domain services 为空 | 预留了 `RepoCatalogDomainService` 但无实现，选择与验证逻辑尚未从 task 侧迁出 |
| `TaskReposApplicationService` 中 `git_client` 为 Optional | 当 CLI 未注入时克隆/状态操作静默返回错误 |

## 后续候选

以下按大致优先级排列，不代表承诺顺序。

### 默认 editor 发现

当前 `open` 命令必须显式传 `--editor`。应实现默认编辑器自动发现：按平台读取环境变量或已知配置路径，在未指定时自动选择。

### 初始 skill mount

`task create --skills` 参数已打通：服务层不再拦截，CLI 在任务创建后自动挂载指定技能。但挂载要求技能已预先通过 `skill import` 导入到注册表，否则 mount 报 "skill not found"。后续可优化为创建时自动确保技能可用。

### repo-group 在 task create 阶段的 enrichment

`task create --repo-groups` 已正常接受参数并存储。剩余缺口是将 repo-group 在创建时展开为具体 repo 列表，而非后续单独操作。

### runtime 实现补全

macOS 和 Windows 的 `SkillRuntime` 均为空 placeholder 结构体。应实现 `prepare`、`use_skill` 和 `check_prepare_status` 的具体逻辑，使技能生命周期可执行。

### JSON 输出 / 机器可读接口

`json.rs` 明确标记为 V1 范围外。后续应提供 JSON 输出模式，支持机器解析和脚本集成。

### skill override

`override_skill` 当前直接返回 `AdapterUnavailable`，应实现技能挂载的局部覆盖能力。

### git ahead/behind 真实计算

`get_repo_status` 应通过 `git rev-list --count` 计算本地与远程的真实差异，替换当前的恒 0 值。

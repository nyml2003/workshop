# 现状与演进

本文档记录 workc 的当前完成度，不承诺任何时间表。

## 已完成

| 功能 | 说明 |
|---|---|
| 任务 CRUD | 创建（含 ID 生成和 slug 唯一性校验）、列表（按状态/标签过滤、按活动时间排序）、用编辑器打开、关闭 |
| 仓库目录 | `repo add` / `repo list`，`repo-group add` / `repo-group list`，持久化到 `repos/catalog.toml` 和 `repos/groups.toml` |
| 任务仓库关联 | `task repos set/add/remove`，支持通过 repo-group 批量关联；`clone`（含 `--missing`、`--dry-run`）和 `status` 检查 |
| 技能注册 | `skill import` 支持 git、local、archive 三种来源；`show` 和 `versions` 查看；持久化到 `skills/registry/` |
| 技能挂载 | `skill mount/unmount` 将技能关联到任务；版本更新检查（`check_skill_updates`） |
| 知识候选 | 在任务下创建、查看、编辑、删除候选；持久化到 `tasks/<task-id>/knowledge-candidates/` |
| 知识条目 | 全局知识的 CRUD；`promote` 将候选提升为全局条目 |
| 四层架构骨架 | 领域层、应用层（含 ports）、基础设施层（FS 仓储、git adapter、editor adapter、system clock）、CLI 层 |
| 跨平台 editor 启动 | macOS（Cursor/VS Code）和 Windows（Cursor/VS Code） |
| 文本和 JSON 输出 | 所有命令都有对应的文本和 JSON 渲染函数 |
| 测试覆盖 | 11 个应用层测试 + 5 个基础设施层测试 + 47 项冒烟测试 |

## 后续候选

以下按大致优先级排列，不代表承诺顺序。

### 默认 editor 发现

`open` 命令必须显式传 `--editor`。应实现默认编辑器自动发现：按平台读取环境变量或已知配置路径，在未指定时自动选择。

### repo-group 在 task create 阶段的 enrichment

`task create --repo-groups` 已接受参数并存储。剩余缺口是将 repo-group 在创建时展开为具体 repo 列表，而非后续单独操作。

### skill runtime 实现

`SkillRuntime` 在 macOS 和 Windows 上为占位实现。`prepare`、`use_skill` 和 `check_prepare_status` 方法返回 `AdapterUnavailable`。应实现具体逻辑使技能生命周期可执行，并暴露为 CLI 子命令。

### git ahead/behind 真实计算

`get_repo_status` 应通过 `git rev-list --count` 计算本地与远程的真实差异，替换当前的恒 0 值。

### skill override

`override_skill` 方法直接返回 `AdapterUnavailable`，应实现技能挂载的局部覆盖能力。

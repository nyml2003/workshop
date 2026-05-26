use anyhow::{Result, anyhow};
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use workc_application::ports::Clock;
use workc_application::task::{
    ApplicationTaskStatus, CloseTaskCommand, CreateTaskCommand, DefaultTaskApplicationService,
    ListTasksQuery, OpenTaskCommand, TaskApplicationService, TaskSlug,
};
use workc_application::task_skills::{
    DefaultTaskSkillsApplicationService, MountSkillCommand, TaskSkillsApplicationService,
};
use workc_domain::workspace::{WorkspaceEntry, WorkspaceRegistryRepository, WorkspaceStatus};
#[cfg(target_os = "linux")]
use workc_infrastructure::editor::linux::LinuxEditorLauncher;
#[cfg(target_os = "macos")]
use workc_infrastructure::editor::macos::MacOsEditorLauncher;
#[cfg(target_os = "windows")]
use workc_infrastructure::editor::windows::WindowsEditorLauncher;
use workc_infrastructure::fs::{
    FsSkillRegistryRepository, FsTaskRepository, FsTaskSkillMountRepository,
    FsWorkspaceRegistryRepository,
};
use workc_infrastructure::time::system_clock::SystemClock;

use super::context::CliContext;
use super::repo::{RepoCommand, RepoGroupCommand, TaskReposCommand};
use crate::presenters::{self, Presenter};

#[derive(Parser, Debug)]
#[command(name = "workc")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    List(ListCommand),
    Open(OpenCommand),
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    #[command(name = "repo-group")]
    RepoGroup {
        #[command(subcommand)]
        command: RepoGroupCommand,
    },
    Skill {
        #[command(subcommand)]
        command: super::skill::SkillCommand,
    },
    Knowledge {
        #[command(subcommand)]
        command: super::knowledge::KnowledgeCommand,
    },
    Task(TaskCommand),
}

#[derive(Args, Debug)]
pub struct ListCommand {
    #[arg(long)]
    pub status: Option<TaskStatusArg>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Args, Debug)]
pub struct OpenCommand {
    pub task: String,
    #[arg(long)]
    pub editor: Option<EditorArg>,
}

#[derive(Args, Debug)]
pub struct TaskCommand {
    #[command(subcommand)]
    pub command: TaskSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum TaskSubcommand {
    Create(CreateTaskArgs),
    Close(CloseTaskArgs),
    Repos {
        #[command(subcommand)]
        command: TaskReposCommand,
    },
}

#[derive(Args, Debug)]
pub struct CreateTaskArgs {
    #[arg(long)]
    pub slug: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub template: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long = "source-brief")]
    pub source_brief: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    #[arg(long = "repo-groups", value_delimiter = ',')]
    pub selected_repo_groups: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub repos: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub skills: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CloseTaskArgs {
    pub task_id: String,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum EditorArg {
    Cursor,
    Vscode,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TaskStatusArg {
    Draft,
    Active,
    Closed,
    Archived,
}

fn task_service(ctx: &CliContext) -> Result<DefaultTaskApplicationService> {
    Ok(DefaultTaskApplicationService::new(
        ctx.workspace_root.clone(),
        Box::new(FsTaskRepository::new(
            ctx.workspace_root.clone(),
            ctx.fs.clone_box(),
        )),
        Box::new(SystemClock),
        Box::new(default_editor_launcher()),
    ))
}

fn task_service_for_root(
    root: Utf8PathBuf,
    ctx: &CliContext,
) -> Result<DefaultTaskApplicationService> {
    Ok(DefaultTaskApplicationService::new(
        root.clone(),
        Box::new(FsTaskRepository::new(root, ctx.fs.clone_box())),
        Box::new(SystemClock),
        Box::new(default_editor_launcher()),
    ))
}

fn workspace_registry(ctx: &CliContext) -> FsWorkspaceRegistryRepository {
    FsWorkspaceRegistryRepository::new(ctx.fs.clone_box())
}

fn register_workspace(
    slug: &str,
    title: &str,
    status: WorkspaceStatus,
    ctx: &CliContext,
) -> Result<()> {
    let registry = workspace_registry(ctx);
    let mut entries = registry.load()?;
    let now = SystemClock.now();

    if let Some(existing) = entries.iter_mut().find(|e| e.path == ctx.workspace_root) {
        existing.slug = TaskSlug::from(slug);
        existing.title = title.to_owned();
        existing.status = status;
        existing.last_activity_at = Some(now);
    } else {
        entries.push(WorkspaceEntry {
            slug: TaskSlug::from(slug),
            path: ctx.workspace_root.clone(),
            title: title.to_owned(),
            status,
            last_activity_at: Some(now),
        });
    }

    registry.save(&entries)?;
    Ok(())
}

fn update_workspace_status(status: WorkspaceStatus, ctx: &CliContext) -> Result<()> {
    let registry = workspace_registry(ctx);
    let mut entries = registry.load()?;
    if let Some(entry) = entries.iter_mut().find(|e| e.path == ctx.workspace_root) {
        entry.status = status;
        entry.last_activity_at = Some(SystemClock.now());
        registry.save(&entries)?;
    }
    Ok(())
}

fn task_skill_service_for_create(ctx: &CliContext) -> Result<DefaultTaskSkillsApplicationService> {
    Ok(DefaultTaskSkillsApplicationService::new(
        Box::new(FsTaskRepository::new(
            ctx.workspace_root.clone(),
            ctx.fs.clone_box(),
        )),
        Box::new(FsTaskSkillMountRepository::new(
            ctx.workspace_root.clone(),
            ctx.fs.clone_box(),
        )),
        Box::new(FsSkillRegistryRepository::new(ctx.fs.clone_box())),
        Box::new(SystemClock),
    ))
}

#[cfg(target_os = "windows")]
fn default_editor_launcher() -> WindowsEditorLauncher {
    WindowsEditorLauncher::new()
}

#[cfg(target_os = "macos")]
fn default_editor_launcher() -> MacOsEditorLauncher {
    MacOsEditorLauncher::new()
}

#[cfg(target_os = "linux")]
fn default_editor_launcher() -> LinuxEditorLauncher {
    LinuxEditorLauncher::new()
}

fn to_editor_name(value: EditorArg) -> &'static str {
    match value {
        EditorArg::Cursor => "cursor",
        EditorArg::Vscode => "vscode",
    }
}

fn to_task_status(value: TaskStatusArg) -> ApplicationTaskStatus {
    match value {
        TaskStatusArg::Draft => ApplicationTaskStatus::Draft,
        TaskStatusArg::Active => ApplicationTaskStatus::Active,
        TaskStatusArg::Closed => ApplicationTaskStatus::Closed,
        TaskStatusArg::Archived => ApplicationTaskStatus::Archived,
    }
}

pub fn run() -> Result<String> {
    let cli = Cli::parse();
    let ctx = CliContext::production()?;
    run_command(cli, ctx)
}

pub fn run_command(cli: Cli, ctx: CliContext) -> Result<String> {
    let presenter: Box<dyn Presenter> = if cli.json {
        Box::new(presenters::json::JsonPresenter)
    } else {
        Box::new(presenters::TextPresenter)
    };

    let service = task_service(&ctx)?;

    match cli.command {
        Command::List(command) => {
            let items = service.list_tasks(ListTasksQuery {
                status: command.status.map(to_task_status),
                tag: command.tag,
                limit: command.limit,
            })?;
            Ok(presenter.render_task_list(&items))
        }
        Command::Open(command) => {
            let editor = command.editor.clone().map(|e| to_editor_name(e).to_owned());
            let registry = workspace_registry(&ctx);
            let entries = registry.load()?;
            let workspace_service = entries
                .iter()
                .find(|e| e.slug.as_str() == command.task)
                .map(|e| task_service_for_root(e.path.clone(), &ctx))
                .transpose()?
                .ok_or_else(|| anyhow!("workspace not found: {}", command.task))?;

            workspace_service.open_task(OpenTaskCommand {
                task: command.task.clone(),
                editor,
            })?;
            let editor_name = command
                .editor
                .map(|value| match value {
                    EditorArg::Cursor => "cursor",
                    EditorArg::Vscode => "vscode",
                })
                .unwrap_or("unknown");
            Ok(presenter.render_task_opened(&command.task, editor_name))
        }
        Command::Repo { command } => super::repo::run_repo(command, presenter.as_ref(), &ctx),
        Command::RepoGroup { command } => {
            super::repo::run_repo_group(command, presenter.as_ref(), &ctx)
        }
        Command::Skill { command } => super::skill::run(command, presenter.as_ref(), &ctx),
        Command::Knowledge { command } => super::knowledge::run(command, presenter.as_ref(), &ctx),
        Command::Task(task_command) => match task_command.command {
            TaskSubcommand::Create(command) => {
                let result = service.create_task(CreateTaskCommand {
                    slug: command.slug,
                    title: command.title,
                    template: command.template,
                    description: command.description,
                    source_brief: command.source_brief,
                    tags: command.tags.clone(),
                    selected_repo_groups: command.selected_repo_groups,
                    repos: command.repos,
                    initial_skills: command.skills.clone(),
                })?;

                if !command.skills.is_empty() {
                    let skill_service = task_skill_service_for_create(&ctx)?;
                    for skill_id in &command.skills {
                        skill_service.mount_skill(MountSkillCommand {
                            task_id: result.task_id.clone(),
                            skill_id: skill_id.clone(),
                            version: None,
                        })?;
                    }
                }

                register_workspace(&result.slug, &result.title, WorkspaceStatus::Active, &ctx)?;

                Ok(presenter.render_task_created(&result))
            }
            TaskSubcommand::Close(command) => {
                service.close_task(CloseTaskCommand {
                    task_id: command.task_id,
                })?;
                update_workspace_status(WorkspaceStatus::Closed, &ctx)?;
                Ok(presenter.render_message("Closed task"))
            }
            TaskSubcommand::Repos { command } => {
                super::repo::run_task_repos(command, presenter.as_ref(), &ctx)
            }
        },
    }
}

#[cfg(test)]
mod dry_run_tests {
    use camino::Utf8PathBuf;
    use clap::Parser;
    use workc_infrastructure::fs::{FileSystem, MemoryFileSystem};

    use super::*;
    use crate::cli::context::CliContext;

    fn test_context(memfs: &MemoryFileSystem) -> CliContext {
        CliContext::new(memfs.clone_box(), Utf8PathBuf::from("/test"))
    }

    #[test]
    fn task_create_and_list() {
        let memfs = MemoryFileSystem::new();
        let ctx = test_context(&memfs);

        let cli = Cli::parse_from([
            "workc",
            "task",
            "create",
            "--slug",
            "my-task",
            "--title",
            "Smoke Test",
            "--template",
            "default",
        ]);
        let output = run_command(cli, ctx).unwrap();
        assert!(output.contains("Created task"));

        let ops = memfs.ops();
        assert!(
            ops.iter()
                .any(|c| matches!(c, workc_infrastructure::fs::FsCall::CreateDirAll(_)))
        );
        assert!(
            ops.iter()
                .any(|c| matches!(c, workc_infrastructure::fs::FsCall::Write(..)))
        );

        let toml = memfs
            .read_to_string(&Utf8PathBuf::from("/test/.workc.toml"))
            .unwrap();
        assert!(toml.contains("my-task"));
        assert!(toml.contains("Smoke Test"));
    }

    #[test]
    fn task_close() {
        let memfs = MemoryFileSystem::new();

        let cli = Cli::parse_from([
            "workc",
            "task",
            "create",
            "--slug",
            "to-close",
            "--title",
            "Close Me",
            "--template",
            "default",
        ]);
        run_command(cli, test_context(&memfs)).unwrap();

        let cli = Cli::parse_from(["workc", "task", "close", "to-close"]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("Closed task"));

        let toml = memfs
            .read_to_string(&Utf8PathBuf::from("/test/.workc.toml"))
            .unwrap();
        assert!(toml.contains("closed"));
    }

    #[test]
    fn repo_add_and_list() {
        let memfs = MemoryFileSystem::new();
        let cli = Cli::parse_from([
            "workc",
            "repo",
            "add",
            "test-repo",
            "https://github.com/test/repo.git",
        ]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("test-repo"));

        let ops = memfs.ops();
        assert!(
            ops.iter()
                .any(|c| matches!(c, workc_infrastructure::fs::FsCall::CreateDirAll(_)))
        );
        assert!(
            ops.iter()
                .any(|c| matches!(c, workc_infrastructure::fs::FsCall::Write(..)))
        );
    }

    #[test]
    fn knowledge_candidate_promote() {
        let memfs = MemoryFileSystem::new();
        let cli = Cli::parse_from([
            "workc",
            "knowledge",
            "candidate",
            "create",
            "my-task",
            "c1",
            "--title",
            "Test Knowledge",
            "--source",
            "docs/readme",
        ]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("Test Knowledge"));

        let cli = Cli::parse_from(["workc", "knowledge", "promote", "my-task", "c1", "k1"]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("Test Knowledge"));
    }

    #[test]
    fn skill_import_and_mount() {
        let memfs = MemoryFileSystem::new();

        let skill_dir = Utf8PathBuf::from("/test/test-skill");
        memfs.create_dir_all(&skill_dir).unwrap();
        memfs
            .write(
                &skill_dir.join("skill.toml"),
                r#"name = "test-skill"
version = "0.1.0"
description = "Test skill"
"#,
            )
            .unwrap();

        let cli = Cli::parse_from([
            "workc",
            "skill",
            "import",
            "local",
            "test-skill",
            "--name",
            "test-skill-import",
            "--version",
            "0.1.0",
        ]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("Imported skill source"));

        let cli = Cli::parse_from([
            "workc",
            "task",
            "create",
            "--slug",
            "skill-task",
            "--title",
            "Skill Test",
            "--template",
            "default",
            "--skills",
            "test-skill-import",
        ]);
        let output = run_command(cli, test_context(&memfs)).unwrap();
        assert!(output.contains("Created task"));
    }

    #[test]
    fn open_without_editor_fails() {
        let memfs = MemoryFileSystem::new();
        let cli = Cli::parse_from([
            "workc",
            "task",
            "create",
            "--slug",
            "editor-test",
            "--title",
            "Editor",
            "--template",
            "default",
        ]);
        run_command(cli, test_context(&memfs)).unwrap();

        let cli = Cli::parse_from(["workc", "open", "editor-test"]);
        let result = run_command(cli, test_context(&memfs));
        assert!(result.is_err());
    }
}

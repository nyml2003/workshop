use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use super::knowledge::KnowledgeCommand;
use super::skill::SkillCommand;
use workc_application::ports::EditorKind;
use workc_application::task::{
    ApplicationTaskStatus, CloseTaskCommand, CreateTaskCommand, DefaultTaskApplicationService, ListTasksQuery, OpenTaskCommand,
    TaskApplicationService, TaskRef,
};
#[cfg(target_os = "windows")]
use workc_infrastructure::editor::windows::WindowsEditorLauncher;
#[cfg(target_os = "macos")]
use workc_infrastructure::editor::macos::MacOsEditorLauncher;
use workc_infrastructure::fs::task_repository::{DefaultTaskIdGenerator, FsTaskRepository};
use workc_infrastructure::time::system_clock::SystemClock;

use crate::presenters::text;
use super::repo::{RepoCommand, RepoGroupCommand, TaskReposCommand};

#[derive(Parser, Debug)]
#[command(name = "workc")]
pub struct Cli {
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
        command: SkillCommand,
    },
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
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

fn workspace_root() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|path| anyhow!("workspace root is not valid UTF-8: {}", path.display()))
}

fn task_service() -> Result<DefaultTaskApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultTaskApplicationService::new(
        workspace_root.clone(),
        Box::new(FsTaskRepository::new(workspace_root)),
        Box::new(SystemClock),
        Box::new(DefaultTaskIdGenerator),
        Box::new(default_editor_launcher()),
    ))
}

#[cfg(target_os = "windows")]
fn default_editor_launcher() -> WindowsEditorLauncher {
    WindowsEditorLauncher
}

#[cfg(target_os = "macos")]
fn default_editor_launcher() -> MacOsEditorLauncher {
    MacOsEditorLauncher
}

fn parse_task_ref(value: &str) -> TaskRef {
    if value.starts_with("task-") {
        TaskRef::Id(value.to_owned())
    } else {
        TaskRef::Slug(value.to_owned())
    }
}

fn to_editor_kind(value: EditorArg) -> EditorKind {
    match value {
        EditorArg::Cursor => EditorKind::Cursor,
        EditorArg::Vscode => EditorKind::VsCode,
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
    let service = task_service()?;

    match cli.command {
        Command::List(command) => {
            let items = service.list_tasks(ListTasksQuery {
                status: command.status.map(to_task_status),
                tag: command.tag,
                limit: command.limit,
            })?;
            Ok(text::render_task_list(&items))
        }
        Command::Open(command) => {
            let editor = command.editor.clone().map(to_editor_kind);
            service.open_task(OpenTaskCommand {
                task: parse_task_ref(&command.task),
                editor,
            })?;
            let editor_name = command
                .editor
                .map(|value| match value {
                    EditorArg::Cursor => "cursor",
                    EditorArg::Vscode => "vscode",
                })
                .unwrap_or("unknown");
            Ok(text::render_task_opened(&command.task, editor_name))
        }
        Command::Repo { command } => super::repo::run_repo(command),
        Command::RepoGroup { command } => super::repo::run_repo_group(command),
        Command::Skill { command } => super::skill::run(command),
        Command::Knowledge { command } => super::knowledge::run(command),
        Command::Task(task_command) => match task_command.command {
            TaskSubcommand::Create(command) => {
                let result = service.create_task(CreateTaskCommand {
                    slug: command.slug,
                    title: command.title,
                    template: command.template,
                    description: command.description,
                    source_brief: command.source_brief,
                    tags: command.tags,
                    selected_repo_groups: command.selected_repo_groups,
                    repos: command.repos,
                    initial_skills: command.skills,
                })?;
                Ok(text::render_task_created(
                    &result.task_id,
                    &result.slug,
                    &result.title,
                    &result.template,
                ))
            }
            TaskSubcommand::Close(command) => {
                service.close_task(CloseTaskCommand {
                    task_id: command.task_id,
                })?;
                Ok("Closed task".to_owned())
            }
            TaskSubcommand::Repos { command } => super::repo::run_task_repos(command),
        },
    }
}

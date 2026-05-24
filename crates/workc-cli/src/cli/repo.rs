use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use workc_application::repo_catalog::{
    AddRepoCommand, AddRepoGroupCommand, DefaultRepoCatalogApplicationService, RepoCatalogApplicationService,
};
use workc_application::task_repos::{
    AddTaskReposCommand, CloneStateFilter, CloneTaskReposCommand, DefaultTaskReposApplicationService, RemoveTaskReposCommand,
    RepoStatusQuery, SetTaskReposCommand, TaskReposApplicationService,
};
use workc_infrastructure::git::command_git_client::CommandGitClient;
use workc_infrastructure::fs::{FsRepoCatalogRepository, FsTaskRepository};
use workc_infrastructure::time::system_clock::SystemClock;

use crate::presenters::Presenter;

#[derive(Subcommand, Debug)]
pub enum RepoCommand {
    Add(RepoAddArgs),
    List,
}

#[derive(Args, Debug)]
pub struct RepoAddArgs {
    pub id: String,
    pub url: String,
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum RepoGroupCommand {
    Add(RepoGroupAddArgs),
    List,
}

#[derive(Args, Debug)]
pub struct RepoGroupAddArgs {
    pub id: String,
    pub repos: String,
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum TaskReposCommand {
    Set(TaskReposSetArgs),
    Add(TaskReposAddArgs),
    Remove(TaskReposRemoveArgs),
    Clone(TaskReposCloneArgs),
    Status(TaskReposStatusArgs),
}

#[derive(Args, Debug)]
pub struct TaskReposSetArgs {
    pub task: String,
    #[arg(long = "repo-groups", value_delimiter = ',')]
    pub repo_groups: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub repos: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TaskReposAddArgs {
    pub task: String,
    pub repos: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TaskReposRemoveArgs {
    pub task: String,
    pub repos: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TaskReposCloneArgs {
    pub task: String,
    #[arg(long = "repo")]
    pub repos: Vec<String>,
    #[arg(long)]
    pub missing: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct TaskReposStatusArgs {
    pub task: String,
    #[arg(long = "repo")]
    pub repos: Vec<String>,
    #[arg(long = "clone-state")]
    pub clone_state: Option<CloneStateArg>,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CloneStateArg {
    Missing,
    Ready,
    Dirty,
    Unknown,
}

fn workspace_root() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|path| anyhow!("workspace root is not valid UTF-8: {}", path.display()))
}

fn repo_catalog_service() -> Result<DefaultRepoCatalogApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultRepoCatalogApplicationService::new(Box::new(
        FsRepoCatalogRepository::new(workspace_root),
    )))
}

fn task_repos_service() -> Result<DefaultTaskReposApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultTaskReposApplicationService::new(
        Box::new(FsTaskRepository::new(workspace_root.clone())),
        Box::new(FsRepoCatalogRepository::new(workspace_root)),
        Box::new(SystemClock),
        Box::new(CommandGitClient),
    ))
}

fn to_clone_state_filter(value: CloneStateArg) -> CloneStateFilter {
    match value {
        CloneStateArg::Missing => CloneStateFilter::Missing,
        CloneStateArg::Ready => CloneStateFilter::Ready,
        CloneStateArg::Dirty => CloneStateFilter::Dirty,
        CloneStateArg::Unknown => CloneStateFilter::Unknown,
    }
}

pub fn run_repo(command: RepoCommand, presenter: &dyn Presenter) -> Result<String> {
    let service = repo_catalog_service()?;
    match command {
        RepoCommand::Add(args) => {
            let repo = service.add_repo(AddRepoCommand {
                id: args.id,
                url: args.url,
                tags: args.tags,
                description: args.description,
            })?;
            Ok(presenter.render_repo_created(&repo))
        }
        RepoCommand::List => {
            let repos = service.list_repos()?;
            Ok(presenter.render_repo_list(&repos))
        }
    }
}

pub fn run_repo_group(command: RepoGroupCommand, presenter: &dyn Presenter) -> Result<String> {
    let service = repo_catalog_service()?;
    match command {
        RepoGroupCommand::Add(args) => {
            let group = service.add_repo_group(AddRepoGroupCommand {
                id: args.id,
                repos: args
                    .repos
                    .split(',')
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| value.trim().to_owned())
                    .collect(),
                tags: args.tags,
                description: args.description,
            })?;
            Ok(presenter.render_repo_group_created(&group))
        }
        RepoGroupCommand::List => {
            let groups = service.list_repo_groups()?;
            Ok(presenter.render_repo_group_list(&groups))
        }
    }
}

pub fn run_task_repos(command: TaskReposCommand, presenter: &dyn Presenter) -> Result<String> {
    let service = task_repos_service()?;
    let result = match command {
        TaskReposCommand::Set(args) => service.set_task_repos(SetTaskReposCommand {
            task_id: args.task,
            selected_repo_groups: args.repo_groups,
            repos: args.repos,
        })?,
        TaskReposCommand::Add(args) => service.add_task_repos(AddTaskReposCommand {
            task_id: args.task,
            repos: args.repos,
        })?,
        TaskReposCommand::Remove(args) => service.remove_task_repos(RemoveTaskReposCommand {
            task_id: args.task,
            repos: args.repos,
        })?,
        TaskReposCommand::Clone(args) => {
            let outcomes = service.clone_task_repos(CloneTaskReposCommand {
                task_id: args.task,
                repos: (!args.repos.is_empty()).then_some(args.repos),
                missing_only: args.missing,
                dry_run: args.dry_run,
            })?;
            return Ok(presenter.render_repo_clone_outcomes(&outcomes));
        }
        TaskReposCommand::Status(args) => {
            let items = service.get_repo_statuses(RepoStatusQuery {
                task_id: args.task,
                repos: (!args.repos.is_empty()).then_some(args.repos),
                clone_state: args.clone_state.map(to_clone_state_filter),
                dry_run: args.dry_run,
            })?;
            return Ok(presenter.render_repo_statuses(&items));
        }
    };

    Ok(presenter.render_task_repos_result(&result))
}

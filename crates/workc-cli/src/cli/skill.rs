use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand, ValueEnum};
use workc_application::skill_registry::{
    ApplicationSkillSourceKind, DefaultSkillRegistryApplicationService, ImportSkillSourceCommand, ImportedSkillDefinition,
    ShowSkillQuery, SkillRegistryApplicationService,
};
use workc_application::task_skills::{
    DefaultTaskSkillsApplicationService, MountSkillCommand, TaskSkillsApplicationService, UnmountSkillCommand,
};
use workc_infrastructure::fs::{FsSkillRegistryRepository, FsTaskRepository, FsTaskSkillMountRepository};
use workc_infrastructure::time::system_clock::SystemClock;

use crate::presenters::text;

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    Import(ImportSkillArgs),
    Show(SkillShowArgs),
    Versions(SkillVersionsArgs),
    Mount(SkillMountArgs),
    Mounts(SkillMountsArgs),
    Unmount(SkillUnmountArgs),
}

#[derive(Args, Debug)]
pub struct ImportSkillArgs {
    pub kind: SkillSourceKindArg,
    pub location: String,
    #[arg(long = "name")]
    pub source_id: String,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long = "skill-id")]
    pub skill_id: Option<String>,
}

#[derive(Args, Debug)]
pub struct SkillShowArgs {
    pub skill_id: String,
}

#[derive(Args, Debug)]
pub struct SkillVersionsArgs {
    pub skill_id: String,
}

#[derive(Args, Debug)]
pub struct SkillMountArgs {
    pub task: String,
    pub skill_id: String,
    pub version: Option<String>,
}

#[derive(Args, Debug)]
pub struct SkillMountsArgs {
    pub task: String,
}

#[derive(Args, Debug)]
pub struct SkillUnmountArgs {
    pub task: String,
    pub mount_id: String,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SkillSourceKindArg {
    Git,
    Local,
    Archive,
}

fn workspace_root() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|path| anyhow!("workspace root is not valid UTF-8: {}", path.display()))
}

fn registry_service() -> Result<DefaultSkillRegistryApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultSkillRegistryApplicationService::new(
        Box::new(FsSkillRegistryRepository::new(workspace_root)),
        Box::new(SystemClock),
    ))
}

fn task_skill_service() -> Result<DefaultTaskSkillsApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultTaskSkillsApplicationService::new(
        Box::new(FsTaskRepository::new(workspace_root.clone())),
        Box::new(FsTaskSkillMountRepository::new(workspace_root.clone())),
        Box::new(FsSkillRegistryRepository::new(workspace_root)),
        Box::new(SystemClock),
        None,
    ))
}

fn to_source_kind(value: SkillSourceKindArg) -> ApplicationSkillSourceKind {
    match value {
        SkillSourceKindArg::Git => ApplicationSkillSourceKind::Git,
        SkillSourceKindArg::Local => ApplicationSkillSourceKind::Local,
        SkillSourceKindArg::Archive => ApplicationSkillSourceKind::Archive,
    }
}

pub fn run(command: SkillCommand) -> Result<String> {
    match command {
        SkillCommand::Import(args) => {
            let service = registry_service()?;
            service.import_source(ImportSkillSourceCommand {
                source_id: args.source_id.clone(),
                kind: to_source_kind(args.kind),
                location: args.location,
                reference: args.version.clone(),
                skills: args
                    .skill_id
                    .map(|skill_id| {
                        vec![ImportedSkillDefinition {
                            id: skill_id,
                            versions: args.version.clone().into_iter().collect(),
                            latest: args.version,
                        }]
                    })
                    .unwrap_or_default(),
            })?;
            Ok(format!("Imported skill source {}", args.source_id))
        }
        SkillCommand::Show(args) => {
            let service = registry_service()?;
            let skill = service.show_skill(ShowSkillQuery { skill_id: args.skill_id })?;
            Ok(skill
                .map(|item| text::render_skill_summary(&item))
                .unwrap_or_else(|| "Skill not found.".to_owned()))
        }
        SkillCommand::Versions(args) => {
            let service = registry_service()?;
            let versions = service.list_skill_versions(ShowSkillQuery { skill_id: args.skill_id })?;
            Ok(text::render_skill_versions(&versions))
        }
        SkillCommand::Mount(args) => {
            let service = task_skill_service()?;
            let summary = service.mount_skill(MountSkillCommand {
                task_id: args.task,
                skill_id: args.skill_id,
                version: args.version,
            })?;
            Ok(text::render_skill_mount(&summary))
        }
        SkillCommand::Mounts(args) => {
            let service = task_skill_service()?;
            if !args.task.starts_with("task-") {
                return Err(anyhow!("skill mounts currently requires a task-id"));
            }
            let mounts = service.list_mounts(&workc_application::task::TaskId::from(args.task.as_str()))?;
            Ok(text::render_skill_mounts(&mounts))
        }
        SkillCommand::Unmount(args) => {
            let service = task_skill_service()?;
            service.unmount_skill(UnmountSkillCommand {
                task_id: args.task,
                mount_id: args.mount_id,
            })?;
            Ok("Unmounted skill".to_owned())
        }
    }
}

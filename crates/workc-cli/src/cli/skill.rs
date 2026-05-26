use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use workc_application::skill_registry::{
    ApplicationSkillSourceKind, DefaultSkillRegistryApplicationService, ImportSkillSourceCommand,
    ImportedSkillDefinition, ShowSkillQuery, SkillRegistryApplicationService,
};
use workc_application::task::TaskSlug;
use workc_application::task_skills::{
    CheckSkillUpdatesQuery, DefaultTaskSkillsApplicationService, MountSkillCommand,
    TaskSkillsApplicationService, UnmountSkillCommand,
};
use workc_infrastructure::fs::{
    FsSkillRegistryRepository, FsTaskRepository, FsTaskSkillMountRepository,
};
use workc_infrastructure::time::system_clock::SystemClock;

use super::context::CliContext;
use crate::presenters::Presenter;

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    Import(ImportSkillArgs),
    Show(SkillShowArgs),
    Versions(SkillVersionsArgs),
    Mount(SkillMountArgs),
    Mounts(SkillMountsArgs),
    Unmount(SkillUnmountArgs),
    CheckUpdates(SkillCheckUpdatesArgs),
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

#[derive(Args, Debug)]
pub struct SkillCheckUpdatesArgs {
    pub task: String,
    #[arg(long)]
    pub mount_id: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SkillSourceKindArg {
    Git,
    Local,
    Archive,
}

fn registry_service(ctx: &CliContext) -> Result<DefaultSkillRegistryApplicationService> {
    Ok(DefaultSkillRegistryApplicationService::new(
        Box::new(FsSkillRegistryRepository::new(ctx.fs.clone_box())),
        Box::new(SystemClock),
    ))
}

fn task_skill_service(ctx: &CliContext) -> Result<DefaultTaskSkillsApplicationService> {
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

fn to_source_kind(value: SkillSourceKindArg) -> ApplicationSkillSourceKind {
    match value {
        SkillSourceKindArg::Git => ApplicationSkillSourceKind::Git,
        SkillSourceKindArg::Local => ApplicationSkillSourceKind::Local,
        SkillSourceKindArg::Archive => ApplicationSkillSourceKind::Archive,
    }
}

pub fn run(command: SkillCommand, presenter: &dyn Presenter, ctx: &CliContext) -> Result<String> {
    match command {
        SkillCommand::Import(args) => {
            let reg = FsSkillRegistryRepository::new(ctx.fs.clone_box());
            if matches!(args.kind, SkillSourceKindArg::Local) {
                let src_dir = ctx.workspace_root.join(&args.location);
                let version = args.version.as_deref().unwrap_or("latest");
                reg.cache_local_source(&args.source_id, version, &src_dir)?;
            }
            let service = registry_service(ctx)?;
            service.import_source(ImportSkillSourceCommand {
                source_id: args.source_id.clone(),
                kind: to_source_kind(args.kind),
                location: args.location,
                reference: args.version.clone(),
                skills: {
                    let version = args.version.clone();
                    let versions: Vec<String> = version.clone().into_iter().collect();
                    let skill_id = args.skill_id.clone().unwrap_or(args.source_id.clone());
                    vec![ImportedSkillDefinition {
                        id: skill_id,
                        versions,
                        latest: version,
                    }]
                },
            })?;
            Ok(presenter.render_message(&format!("Imported skill source {}", args.source_id)))
        }
        SkillCommand::Show(args) => {
            let service = registry_service(ctx)?;
            let skill = service.show_skill(ShowSkillQuery {
                skill_id: args.skill_id,
            })?;
            Ok(skill
                .map(|item| presenter.render_skill_summary(&item))
                .unwrap_or_else(|| presenter.render_message("Skill not found.")))
        }
        SkillCommand::Versions(args) => {
            let service = registry_service(ctx)?;
            let versions = service.list_skill_versions(ShowSkillQuery {
                skill_id: args.skill_id,
            })?;
            Ok(presenter.render_skill_versions(&versions))
        }
        SkillCommand::Mount(args) => {
            let service = task_skill_service(ctx)?;
            let summary = service.mount_skill(MountSkillCommand {
                task_id: args.task,
                skill_id: args.skill_id.clone(),
                version: args.version,
            })?;

            let reg = FsSkillRegistryRepository::new(ctx.fs.clone_box());
            let version = summary.version.as_str();
            let cache_dir = reg.cache_dir(&args.skill_id, version);
            if ctx.fs.exists(&cache_dir) {
                for dir in &[".opencode", ".cursor", ".agents", ".claude", ".codex"] {
                    let mount_dir = ctx
                        .workspace_root
                        .join(dir)
                        .join("skills")
                        .join(&args.skill_id);
                    ctx.fs.create_dir_all(&mount_dir)?;
                    ctx.fs.copy_dir(&cache_dir, &mount_dir)?;
                }
            }

            Ok(presenter.render_skill_mount(&summary))
        }
        SkillCommand::Mounts(args) => {
            let service = task_skill_service(ctx)?;
            let mounts = service.list_mounts(&TaskSlug::from(args.task.as_str()))?;
            Ok(presenter.render_skill_mounts(&mounts))
        }
        SkillCommand::Unmount(args) => {
            let service = task_skill_service(ctx)?;
            service.unmount_skill(UnmountSkillCommand {
                task_id: args.task,
                mount_id: args.mount_id,
            })?;
            Ok(presenter.render_message("Unmounted skill"))
        }
        SkillCommand::CheckUpdates(args) => {
            let service = task_skill_service(ctx)?;
            let updates = service.check_skill_updates(CheckSkillUpdatesQuery {
                task_id: args.task,
                mount_id: args.mount_id,
            })?;
            Ok(presenter.render_skill_updates(&updates))
        }
    }
}

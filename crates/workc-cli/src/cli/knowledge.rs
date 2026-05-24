use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};
use workc_application::knowledge::{
    CreateKnowledgeCandidateCommand, DefaultKnowledgeApplicationService, DeleteKnowledgeCandidateCommand, DeleteKnowledgeCommand,
    KnowledgeApplicationService, ListKnowledgeCandidatesQuery, ListKnowledgeQuery, PromoteKnowledgeCommand, ShowKnowledgeCandidateQuery,
    ShowKnowledgeQuery, UpdateKnowledgeCandidateMetaCommand, UpdateKnowledgeMetaCommand,
};
use workc_infrastructure::fs::knowledge_repository::FsKnowledgeRepository;
use workc_infrastructure::time::system_clock::SystemClock;

use crate::presenters::text;

#[derive(Subcommand, Debug)]
pub enum KnowledgeCommand {
    Candidate {
        #[command(subcommand)]
        command: KnowledgeCandidateCommand,
    },
    List,
    Show(KnowledgeShowArgs),
    UpdateMeta(KnowledgeUpdateMetaArgs),
    Delete(KnowledgeDeleteArgs),
    Promote(KnowledgePromoteArgs),
}

#[derive(Subcommand, Debug)]
pub enum KnowledgeCandidateCommand {
    Create(KnowledgeCandidateCreateArgs),
    List(KnowledgeCandidateListArgs),
    Show(KnowledgeCandidateShowArgs),
    UpdateMeta(KnowledgeCandidateUpdateMetaArgs),
    Delete(KnowledgeCandidateDeleteArgs),
}

#[derive(Args, Debug)]
pub struct KnowledgeCandidateCreateArgs {
    pub task_id: String,
    pub candidate_id: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Vec<String>,
    #[arg(long = "source", value_delimiter = ',')]
    pub sources: Vec<String>,
}

#[derive(Args, Debug)]
pub struct KnowledgeCandidateListArgs {
    pub task_id: String,
}

#[derive(Args, Debug)]
pub struct KnowledgeCandidateShowArgs {
    pub task_id: String,
    pub candidate_id: String,
}

#[derive(Args, Debug)]
pub struct KnowledgeCandidateUpdateMetaArgs {
    pub task_id: String,
    pub candidate_id: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct KnowledgeCandidateDeleteArgs {
    pub task_id: String,
    pub candidate_id: String,
}

#[derive(Args, Debug)]
pub struct KnowledgeShowArgs {
    pub knowledge_id: String,
}

#[derive(Args, Debug)]
pub struct KnowledgeUpdateMetaArgs {
    pub knowledge_id: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct KnowledgeDeleteArgs {
    pub knowledge_id: String,
}

#[derive(Args, Debug)]
pub struct KnowledgePromoteArgs {
    pub task_id: String,
    pub candidate_id: String,
    pub knowledge_id: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub category: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
}

fn workspace_root() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|path| anyhow!("workspace root is not valid UTF-8: {}", path.display()))
}

fn knowledge_service() -> Result<DefaultKnowledgeApplicationService> {
    let workspace_root = workspace_root()?;
    Ok(DefaultKnowledgeApplicationService::new(
        Box::new(FsKnowledgeRepository::new(workspace_root)),
        Box::new(SystemClock),
    ))
}

pub fn run(command: KnowledgeCommand) -> Result<String> {
    let service = knowledge_service()?;

    match command {
        KnowledgeCommand::Candidate { command } => match command {
            KnowledgeCandidateCommand::Create(args) => {
                let result = service.create_candidate(CreateKnowledgeCandidateCommand {
                    task_id: args.task_id,
                    candidate_id: args.candidate_id,
                    title: args.title,
                    category: args.category,
                    tags: args.tags,
                    source_paths: args.sources,
                })?;
                Ok(text::render_knowledge_detail(&result.candidate))
            }
            KnowledgeCandidateCommand::List(args) => {
                let items = service.list_candidates(ListKnowledgeCandidatesQuery { task_id: args.task_id })?;
                Ok(text::render_knowledge_list(&items))
            }
            KnowledgeCandidateCommand::Show(args) => {
                let item = service.show_candidate(ShowKnowledgeCandidateQuery {
                    task_id: args.task_id,
                    candidate_id: args.candidate_id,
                })?;
                Ok(item
                    .map(|value| text::render_knowledge_detail(&value))
                    .unwrap_or_else(|| "Knowledge candidate not found.".to_owned()))
            }
            KnowledgeCandidateCommand::UpdateMeta(args) => {
                let result = service.update_candidate_meta(UpdateKnowledgeCandidateMetaCommand {
                    task_id: args.task_id,
                    candidate_id: args.candidate_id,
                    title: args.title,
                    category: args.category,
                    tags: args.tags,
                })?;
                Ok(text::render_knowledge_detail(&result.candidate))
            }
            KnowledgeCandidateCommand::Delete(args) => {
                service.delete_candidate(DeleteKnowledgeCandidateCommand {
                    task_id: args.task_id,
                    candidate_id: args.candidate_id,
                })?;
                Ok("Deleted knowledge candidate".to_owned())
            }
        },
        KnowledgeCommand::List => {
            let items = service.list_knowledge(ListKnowledgeQuery)?;
            Ok(text::render_knowledge_list(&items))
        }
        KnowledgeCommand::Show(args) => {
            let item = service.show_knowledge(ShowKnowledgeQuery {
                knowledge_id: args.knowledge_id,
            })?;
            Ok(item
                .map(|value| text::render_knowledge_detail(&value))
                .unwrap_or_else(|| "Knowledge not found.".to_owned()))
        }
        KnowledgeCommand::UpdateMeta(args) => {
            let result = service.update_knowledge_meta(UpdateKnowledgeMetaCommand {
                knowledge_id: args.knowledge_id,
                title: args.title,
                category: args.category,
                tags: args.tags,
            })?;
            Ok(text::render_knowledge_detail(&result.knowledge))
        }
        KnowledgeCommand::Delete(args) => {
            service.delete_knowledge(DeleteKnowledgeCommand {
                knowledge_id: args.knowledge_id,
            })?;
            Ok("Deleted knowledge".to_owned())
        }
        KnowledgeCommand::Promote(args) => {
            let result = service.promote(PromoteKnowledgeCommand {
                task_id: args.task_id,
                candidate_id: args.candidate_id,
                knowledge_id: args.knowledge_id,
                title: args.title,
                category: args.category,
                tags: args.tags,
            })?;
            Ok(text::render_knowledge_detail(&result.knowledge))
        }
    }
}

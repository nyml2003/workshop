use serde::Serialize;

use workc_application::knowledge::KnowledgeObjectSummary;
use workc_application::repo_catalog::{RepoGroupSummary, RepoSummary};
use workc_application::skill_registry::SkillSummary;
use workc_application::task::{CreateTaskResult, TaskListItem};
use workc_application::task_repos::{RepoCloneOutcome, TaskRepoStatusItem, TaskReposResult};
use workc_application::task_skills::{SkillMountSummary, SkillUpdateStatus};

use super::Presenter;

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

pub struct JsonPresenter;

impl Presenter for JsonPresenter {
    fn render_task_list(&self, items: &[TaskListItem]) -> String {
        serde_json::to_string_pretty(items).expect("JSON serialization should not fail")
    }

    fn render_task_created(&self, result: &CreateTaskResult) -> String {
        serde_json::to_string_pretty(result).expect("JSON serialization should not fail")
    }

    fn render_task_opened(&self, task_ref: &str, editor: &str) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "task": task_ref,
            "editor": editor,
            "status": "opened"
        }))
        .expect("JSON serialization should not fail")
    }

    fn render_message(&self, msg: &str) -> String {
        serde_json::to_string_pretty(&MessageResponse {
            message: msg.to_owned(),
        })
        .expect("JSON serialization should not fail")
    }

    fn render_repo_list(&self, repos: &[RepoSummary]) -> String {
        serde_json::to_string_pretty(repos).expect("JSON serialization should not fail")
    }

    fn render_repo_created(&self, repo: &RepoSummary) -> String {
        serde_json::to_string_pretty(repo).expect("JSON serialization should not fail")
    }

    fn render_repo_group_list(&self, groups: &[RepoGroupSummary]) -> String {
        serde_json::to_string_pretty(groups).expect("JSON serialization should not fail")
    }

    fn render_repo_group_created(&self, group: &RepoGroupSummary) -> String {
        serde_json::to_string_pretty(group).expect("JSON serialization should not fail")
    }

    fn render_task_repos_result(&self, result: &TaskReposResult) -> String {
        serde_json::to_string_pretty(result).expect("JSON serialization should not fail")
    }

    fn render_repo_clone_outcomes(&self, outcomes: &[RepoCloneOutcome]) -> String {
        serde_json::to_string_pretty(outcomes).expect("JSON serialization should not fail")
    }

    fn render_repo_statuses(&self, items: &[TaskRepoStatusItem]) -> String {
        serde_json::to_string_pretty(items).expect("JSON serialization should not fail")
    }

    fn render_knowledge_list(&self, items: &[KnowledgeObjectSummary]) -> String {
        serde_json::to_string_pretty(items).expect("JSON serialization should not fail")
    }

    fn render_knowledge_detail(&self, item: &KnowledgeObjectSummary) -> String {
        serde_json::to_string_pretty(item).expect("JSON serialization should not fail")
    }

    fn render_skill_summary(&self, item: &SkillSummary) -> String {
        serde_json::to_string_pretty(item).expect("JSON serialization should not fail")
    }

    fn render_skill_versions(&self, versions: &[String]) -> String {
        serde_json::to_string_pretty(versions).expect("JSON serialization should not fail")
    }

    fn render_skill_mounts(&self, items: &[SkillMountSummary]) -> String {
        serde_json::to_string_pretty(items).expect("JSON serialization should not fail")
    }

    fn render_skill_mount(&self, item: &SkillMountSummary) -> String {
        serde_json::to_string_pretty(item).expect("JSON serialization should not fail")
    }

    fn render_skill_updates(&self, items: &[SkillUpdateStatus]) -> String {
        serde_json::to_string_pretty(items).expect("JSON serialization should not fail")
    }
}

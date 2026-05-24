//! CLI output presenters.

pub mod json;
pub mod text;

use workc_application::knowledge::KnowledgeObjectSummary;
use workc_application::repo_catalog::{RepoGroupSummary, RepoSummary};
use workc_application::skill_registry::SkillSummary;
use workc_application::task::{CreateTaskResult, TaskListItem};
use workc_application::task_repos::{RepoCloneOutcome, TaskRepoStatusItem, TaskReposResult};
use workc_application::task_skills::{SkillMountSummary, SkillUpdateStatus};

pub trait Presenter {
    fn render_task_list(&self, items: &[TaskListItem]) -> String;
    fn render_task_created(&self, result: &CreateTaskResult) -> String;
    fn render_task_opened(&self, task_ref: &str, editor: &str) -> String;
    fn render_message(&self, msg: &str) -> String;
    fn render_repo_list(&self, repos: &[RepoSummary]) -> String;
    fn render_repo_created(&self, repo: &RepoSummary) -> String;
    fn render_repo_group_list(&self, groups: &[RepoGroupSummary]) -> String;
    fn render_repo_group_created(&self, group: &RepoGroupSummary) -> String;
    fn render_task_repos_result(&self, result: &TaskReposResult) -> String;
    fn render_repo_clone_outcomes(&self, outcomes: &[RepoCloneOutcome]) -> String;
    fn render_repo_statuses(&self, items: &[TaskRepoStatusItem]) -> String;
    fn render_knowledge_list(&self, items: &[KnowledgeObjectSummary]) -> String;
    fn render_knowledge_detail(&self, item: &KnowledgeObjectSummary) -> String;
    fn render_skill_summary(&self, item: &SkillSummary) -> String;
    fn render_skill_versions(&self, versions: &[String]) -> String;
    fn render_skill_mounts(&self, items: &[SkillMountSummary]) -> String;
    fn render_skill_mount(&self, item: &SkillMountSummary) -> String;
    fn render_skill_updates(&self, items: &[SkillUpdateStatus]) -> String;
}

pub struct TextPresenter;

impl Presenter for TextPresenter {
    fn render_task_list(&self, items: &[TaskListItem]) -> String {
        text::render_task_list(items)
    }

    fn render_task_created(&self, result: &CreateTaskResult) -> String {
        text::render_task_created(&result.task_id, &result.slug, &result.title, &result.template)
    }

    fn render_task_opened(&self, task_ref: &str, editor: &str) -> String {
        text::render_task_opened(task_ref, editor)
    }

    fn render_message(&self, msg: &str) -> String {
        msg.to_owned()
    }

    fn render_repo_list(&self, repos: &[RepoSummary]) -> String {
        text::render_repo_list(repos)
    }

    fn render_repo_created(&self, repo: &RepoSummary) -> String {
        text::render_repo_created(repo)
    }

    fn render_repo_group_list(&self, groups: &[RepoGroupSummary]) -> String {
        text::render_repo_group_list(groups)
    }

    fn render_repo_group_created(&self, group: &RepoGroupSummary) -> String {
        text::render_repo_group_created(group)
    }

    fn render_task_repos_result(&self, result: &TaskReposResult) -> String {
        text::render_task_repos_result(result)
    }

    fn render_repo_clone_outcomes(&self, outcomes: &[RepoCloneOutcome]) -> String {
        text::render_repo_clone_outcomes(outcomes)
    }

    fn render_repo_statuses(&self, items: &[TaskRepoStatusItem]) -> String {
        text::render_repo_statuses(items)
    }

    fn render_knowledge_list(&self, items: &[KnowledgeObjectSummary]) -> String {
        text::render_knowledge_list(items)
    }

    fn render_knowledge_detail(&self, item: &KnowledgeObjectSummary) -> String {
        text::render_knowledge_detail(item)
    }

    fn render_skill_summary(&self, item: &SkillSummary) -> String {
        text::render_skill_summary(item)
    }

    fn render_skill_versions(&self, versions: &[String]) -> String {
        text::render_skill_versions(versions)
    }

    fn render_skill_mounts(&self, items: &[SkillMountSummary]) -> String {
        text::render_skill_mounts(items)
    }

    fn render_skill_mount(&self, item: &SkillMountSummary) -> String {
        text::render_skill_mount(item)
    }

    fn render_skill_updates(&self, items: &[SkillUpdateStatus]) -> String {
        text::render_skill_updates(items)
    }
}

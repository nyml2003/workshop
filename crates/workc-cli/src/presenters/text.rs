use workc_application::knowledge::KnowledgeObjectSummary;
use workc_application::repo_catalog::{RepoGroupSummary, RepoSummary};
use workc_application::skill_registry::SkillSummary;
use workc_application::task::TaskListItem;
use workc_application::task_repos::{RepoCloneOutcome, TaskRepoStatusItem, TaskReposResult};
use workc_application::task_skills::{SkillMountSummary, SkillUpdateStatus};

pub fn render_task_created(id: &str, slug: &str, title: &str, template: &str) -> String {
    format!("Created task {id}\n  slug: {slug}\n  title: {title}\n  template: {template}")
}

pub fn render_task_list(items: &[TaskListItem]) -> String {
    if items.is_empty() {
        return "No tasks found.".to_owned();
    }

    items
        .iter()
        .map(|item| {
            let last_activity = item
                .last_activity_at
                .map(|value| value.to_string())
                .unwrap_or_else(|| "never".to_owned());
            format!(
                "{} | {} | {:?} | last_activity_at={}",
                item.slug, item.title, item.status, last_activity
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_task_opened(task_ref: &str, editor: &str) -> String {
    format!("Opened task {task_ref} with {editor}")
}

pub fn render_repo_created(repo: &RepoSummary) -> String {
    format!("Added repo {}\n  url: {}", repo.id, repo.url)
}

pub fn render_repo_list(repos: &[RepoSummary]) -> String {
    if repos.is_empty() {
        return "No repos found.".to_owned();
    }

    repos
        .iter()
        .map(|repo| format!("{} | {} | tags={}", repo.id, repo.url, repo.tags.join(",")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_repo_group_created(group: &RepoGroupSummary) -> String {
    format!(
        "Added repo-group {}\n  repos: {}",
        group.id,
        group.repos.join(",")
    )
}

pub fn render_repo_group_list(groups: &[RepoGroupSummary]) -> String {
    if groups.is_empty() {
        return "No repo groups found.".to_owned();
    }

    groups
        .iter()
        .map(|group| {
            format!(
                "{} | repos={} | tags={}",
                group.id,
                group.repos.join(","),
                group.tags.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_task_repos_result(result: &TaskReposResult) -> String {
    format!(
        "Updated task repos for {}\n  repo_groups: {}\n  repos: {}",
        result.task_id,
        if result.selected_repo_groups.is_empty() {
            "-".to_owned()
        } else {
            result.selected_repo_groups.join(",")
        },
        if result.repos.is_empty() {
            "-".to_owned()
        } else {
            result.repos.join(",")
        }
    )
}

pub fn render_repo_clone_outcomes(outcomes: &[RepoCloneOutcome]) -> String {
    if outcomes.is_empty() {
        return "No repos selected for clone.".to_owned();
    }

    outcomes
        .iter()
        .map(|outcome| {
            let mode = if outcome.dry_run { "dry-run" } else { "real" };
            let result = if outcome.cloned {
                "cloned".to_owned()
            } else {
                outcome
                    .skipped_reason
                    .clone()
                    .unwrap_or_else(|| "skipped".to_owned())
            };
            format!(
                "{} | {} | {} | {}",
                outcome.repo_id, outcome.path, mode, result
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_repo_statuses(items: &[TaskRepoStatusItem]) -> String {
    if items.is_empty() {
        return "No repo statuses found.".to_owned();
    }

    items
        .iter()
        .map(|item| {
            format!(
                "{} | {:?} | branch={} | dirty={} | ahead={} | behind={}",
                item.repo_id,
                item.status.clone_state,
                item.status.branch.clone().unwrap_or_else(|| "-".to_owned()),
                item.status.dirty,
                item.status.ahead,
                item.status.behind
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_knowledge_list(items: &[KnowledgeObjectSummary]) -> String {
    if items.is_empty() {
        return "No knowledge objects found.".to_owned();
    }

    items
        .iter()
        .map(|item| {
            format!(
                "{} | {} | category={} | tags={}",
                item.id,
                item.path,
                item.category.clone().unwrap_or_else(|| "-".to_owned()),
                item.tags.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_knowledge_detail(item: &KnowledgeObjectSummary) -> String {
    format!(
        "Knowledge {}\n  title: {}\n  path: {}\n  category: {}\n  tags: {}\n  sources: {}",
        item.id,
        item.title,
        item.path,
        item.category.clone().unwrap_or_else(|| "-".to_owned()),
        if item.tags.is_empty() {
            "-".to_owned()
        } else {
            item.tags.join(",")
        },
        item.source_count
    )
}

pub fn render_skill_summary(item: &SkillSummary) -> String {
    format!(
        "Skill {}\n  source: {}\n  versions: {}\n  latest: {}",
        item.id,
        item.source,
        if item.versions.is_empty() {
            "-".to_owned()
        } else {
            item.versions.join(",")
        },
        item.latest.clone().unwrap_or_else(|| "-".to_owned())
    )
}

pub fn render_skill_versions(versions: &[String]) -> String {
    if versions.is_empty() {
        return "No versions found.".to_owned();
    }

    versions.join("\n")
}

pub fn render_skill_mounts(items: &[SkillMountSummary]) -> String {
    if items.is_empty() {
        return "No skill mounts found.".to_owned();
    }

    items
        .iter()
        .map(|item| {
            format!(
                "{} | {} | version={} | status={}",
                item.mount_id, item.skill_id, item.version, item.status
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_skill_mount(item: &SkillMountSummary) -> String {
    format!(
        "Mounted skill {}\n  mount_id: {}\n  version: {}\n  source: {}\n  path: {}",
        item.skill_id, item.mount_id, item.version, item.source, item.path
    )
}

pub fn render_skill_updates(items: &[SkillUpdateStatus]) -> String {
    if items.is_empty() {
        return "No skill updates found.".to_owned();
    }

    items
        .iter()
        .map(|item| {
            format!(
                "{} | update_available={} | target_version={}",
                item.mount_id,
                item.update_available,
                item.target_version
                    .clone()
                    .unwrap_or_else(|| "-".to_owned())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_error(message: &str) -> String {
    format!("Error: {message}")
}

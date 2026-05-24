use std::process::Command;

use camino::Utf8Path;
use workc_application::ports::{CloneState, GitClient, GitError, RepoStatus};

pub struct CommandGitClient;

impl CommandGitClient {
    fn ahead_behind(path: &Utf8Path) -> Result<(usize, usize), GitError> {
        let upstream = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("@{u}")
            .output()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;

        if !upstream.status.success() {
            return Ok((0, 0));
        }

        let rev_list = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("rev-list")
            .arg("--count")
            .arg("--left-right")
            .arg("@{upstream}...HEAD")
            .output()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;

        if !rev_list.status.success() {
            return Ok((0, 0));
        }

        let output = String::from_utf8_lossy(&rev_list.stdout).trim().to_owned();
        let parts: Vec<&str> = output.split('\t').collect();
        if parts.len() != 2 {
            return Ok((0, 0));
        }

        let behind = parts[0].trim().parse::<usize>().unwrap_or(0);
        let ahead = parts[1].trim().parse::<usize>().unwrap_or(0);
        Ok((behind, ahead))
    }
}

impl GitClient for CommandGitClient {
    fn clone_repo(&self, path: &Utf8Path, url: &str) -> Result<(), GitError> {
        let parent = path.parent().ok_or(GitError {
            detail: "invalid clone target path".to_owned(),
        })?;
        std::fs::create_dir_all(parent).map_err(|error| GitError {
            detail: error.to_string(),
        })?;

        let status = Command::new("git")
            .arg("clone")
            .arg(url)
            .arg(path.as_str())
            .status()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(GitError {
                detail: format!("git clone failed with status {status}"),
            })
        }
    }

    fn get_repo_status(&self, path: &Utf8Path) -> Result<RepoStatus, GitError> {
        if !path.exists() {
            return Ok(RepoStatus {
                branch: None,
                dirty: false,
                ahead: 0,
                behind: 0,
                clone_state: CloneState::Missing,
            });
        }

        let branch_output = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("branch")
            .arg("--show-current")
            .output()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;
        let status_output = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("status")
            .arg("--porcelain")
            .output()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;

        if !branch_output.status.success() {
            return Err(GitError {
                detail: format!(
                    "git branch --show-current failed with status {}",
                    branch_output.status
                ),
            });
        }

        if !status_output.status.success() {
            return Err(GitError {
                detail: format!(
                    "git status --porcelain failed with status {}",
                    status_output.status
                ),
            });
        }

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_owned();
        let dirty = !String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .is_empty();

        let (behind, ahead) = Self::ahead_behind(path).unwrap_or((0, 0));

        Ok(RepoStatus {
            branch: (!branch.is_empty()).then_some(branch),
            dirty,
            ahead,
            behind,
            clone_state: if dirty {
                CloneState::Dirty
            } else {
                CloneState::Ready
            },
        })
    }

    fn fetch_repo(&self, path: &Utf8Path) -> Result<(), GitError> {
        let status = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("fetch")
            .status()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(GitError {
                detail: format!("git fetch failed with status {status}"),
            })
        }
    }

    fn pull_repo(&self, path: &Utf8Path) -> Result<(), GitError> {
        let status = Command::new("git")
            .arg("-C")
            .arg(path.as_str())
            .arg("pull")
            .status()
            .map_err(|error| GitError {
                detail: error.to_string(),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(GitError {
                detail: format!("git pull failed with status {status}"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use camino::Utf8PathBuf;

    use super::*;

    fn temp_dir() -> Utf8PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!("workc-git-status-{unique}")))
            .unwrap()
    }

    #[test]
    fn get_repo_status_returns_missing_for_absent_path() {
        let git = CommandGitClient;
        let path = Utf8PathBuf::from("tasks/non-existent/repos/api-gateway");

        let status = git.get_repo_status(path.as_path()).unwrap();
        assert_eq!(status.clone_state, CloneState::Missing);
    }

    #[test]
    fn get_repo_status_errors_for_non_repo_directory() {
        let git = CommandGitClient;
        let path = temp_dir();
        std::fs::create_dir_all(&path).unwrap();

        let result = git.get_repo_status(path.as_path());
        assert!(result.is_err());

        std::fs::remove_dir_all(path).unwrap();
    }
}

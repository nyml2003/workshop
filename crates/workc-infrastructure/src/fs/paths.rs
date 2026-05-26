use camino::Utf8PathBuf;
use std::path::PathBuf;

pub fn workc_home() -> Utf8PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    match home {
        Some(dir) => Utf8PathBuf::from(dir).join(".workc"),
        None => {
            let fallback = dirs_fallback();
            eprintln!("warning: HOME/USERPROFILE not set, falling back to {}", fallback.display());
            Utf8PathBuf::from_path_buf(fallback.join(".workc"))
                .unwrap_or_else(|_| Utf8PathBuf::from(".workc"))
        }
    }
}

fn dirs_fallback() -> PathBuf {
    if cfg!(windows) {
        std::env::temp_dir()
    } else {
        PathBuf::from("/tmp")
    }
}

pub fn workc_config_path() -> Utf8PathBuf {
    workc_home().join("config.toml")
}

pub fn workc_repos_root() -> Utf8PathBuf {
    workc_home().join("repos")
}

pub fn workc_skills_root() -> Utf8PathBuf {
    workc_home().join("skills")
}

pub fn workc_skills_registry_root() -> Utf8PathBuf {
    workc_skills_root().join("registry")
}

pub fn workc_skills_cache_root() -> Utf8PathBuf {
    workc_skills_root().join("cache")
}

pub fn workc_knowledge_root() -> Utf8PathBuf {
    workc_home().join("knowledge")
}

pub fn workc_workspaces_path() -> Utf8PathBuf {
    workc_home().join("workspaces.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_end_with_correct_suffixes() {
        assert!(workc_home().as_str().ends_with(".workc"));
        assert!(workc_config_path().as_str().ends_with("config.toml"));
        assert!(workc_repos_root().as_str().ends_with("repos"));
        assert!(workc_skills_root().as_str().ends_with("skills"));
        assert!(workc_skills_registry_root().as_str().ends_with("registry"));
        assert!(workc_skills_cache_root().as_str().ends_with("cache"));
        assert!(workc_knowledge_root().as_str().ends_with("knowledge"));
        assert!(workc_workspaces_path().as_str().ends_with("workspaces.toml"));
    }
}

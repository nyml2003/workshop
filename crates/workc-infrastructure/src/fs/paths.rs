use camino::Utf8PathBuf;

pub fn workc_home() -> Utf8PathBuf {
    let home = if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_owned())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_owned())
    };
    Utf8PathBuf::from(home).join(".workc")
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

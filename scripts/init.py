"""workc initializer — prepares ~/.workc/ global data.

Usage:
  python3 init.py                     # run standalone (from release zip)
  python3 scripts/init.py             # run from source repo

This script:
  1. Ensures ~/.workc/ skeleton exists (idempotent)
  2. Ensures empty TOML registry files exist (idempotent — never overwrites existing data)
  3. Reads ~/.workc/config.toml for knowledge git remote
  4. Git-clones (or pulls) knowledge repo to ~/.workc/knowledge/
"""

import os
import subprocess
import sys

WORKC_HOME = os.path.join(os.path.expanduser("~"), ".workc")


def _ensure_dirs():
    dirs = [
        WORKC_HOME,
        os.path.join(WORKC_HOME, "repos"),
        os.path.join(WORKC_HOME, "skills", "registry"),
        os.path.join(WORKC_HOME, "skills", "cache"),
        os.path.join(WORKC_HOME, "knowledge"),
    ]
    for d in dirs:
        os.makedirs(d, exist_ok=True)


def _ensure_file(path: str, default: str):
    if not os.path.isfile(path):
        with open(path, "w", encoding="utf-8") as f:
            f.write(default)
        print(f"Created {path}")
    else:
        print(f"Exists {path}")


def _ensure_registry_files():
    _ensure_file(
        os.path.join(WORKC_HOME, "skills", "registry", "sources.toml"),
        "# Skill sources\nsources = []\n",
    )
    _ensure_file(
        os.path.join(WORKC_HOME, "skills", "registry", "skills.toml"),
        "# Skill definitions\nskills = []\n",
    )
    _ensure_file(
        os.path.join(WORKC_HOME, "repos", "catalog.toml"),
        "# Repo catalog\nrepos = []\n",
    )
    _ensure_file(
        os.path.join(WORKC_HOME, "repos", "groups.toml"),
        "# Repo groups\ngroups = []\n",
    )
    _ensure_file(
        os.path.join(WORKC_HOME, "workspaces.toml"),
        "# Workspace registry\nworkspaces = []\n",
    )


def _get_knowledge_remote() -> str | None:
    config_path = os.path.join(WORKC_HOME, "config.toml")
    if not os.path.isfile(config_path):
        print("No config.toml found, skipping knowledge clone.")
        return None

    with open(config_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line.startswith("remote") and "=" in line:
                _, value = line.split("=", 1)
                return value.strip().strip('"')
    return None


def _clone_knowledge():
    remote = _get_knowledge_remote()
    if not remote:
        print("Skipping knowledge — no remote configured in config.toml")
        return

    knowledge_dir = os.path.join(WORKC_HOME, "knowledge")
    if os.path.isdir(os.path.join(knowledge_dir, ".git")):
        print(f"Knowledge repo exists, pulling: {knowledge_dir}")
        subprocess.run(["git", "-C", knowledge_dir, "pull"], check=False)
    else:
        print(f"Cloning knowledge: {remote} -> {knowledge_dir}")
        if os.listdir(knowledge_dir):
            print(f"Knowledge dir is not empty but has no .git, skipping.")
            return
        subprocess.run(["git", "clone", remote, knowledge_dir], check=False)


def main():
    print("Initializing workc global data ...")
    _ensure_dirs()
    _ensure_registry_files()
    _clone_knowledge()
    print()
    print("Initialization complete.")
    print(f"Global data: {WORKC_HOME}")
    print()
    print("Next:")
    print("  cd your-project")
    print("  workc task create --slug my-task --title 'My Task' --template default")


if __name__ == "__main__":
    main()

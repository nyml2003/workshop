"""Release script — bump version, commit, tag, push.

Usage:
  python scripts/release.py 0.2.0
"""

import re
import subprocess
import sys
from pathlib import Path


def bump_workspace_version(root: Path, new_version: str):
    cargo_toml = root / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")

    updated = re.sub(
        r'(^\[workspace\.package\]\s*[\s\S]*?)^version\s*=\s*"[^"]*"',
        f'\\1version = "{new_version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )

    if updated == content:
        print("warning: version line not found in [workspace.package]")
    else:
        cargo_toml.write_text(updated, encoding="utf-8")
        print(f"Updated workspace version to {new_version} in Cargo.toml")


def git_status_clean() -> bool:
    result = subprocess.run(
        ["git", "status", "--porcelain"], capture_output=True, text=True
    )
    return result.stdout.strip() == ""


def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/release.py <version>")
        print("Example: python scripts/release.py 0.2.0")
        sys.exit(1)

    version = sys.argv[1]
    if not re.match(r"^\d+\.\d+\.\d+$", version):
        print(f"Invalid version: {version} (expected X.Y.Z)")
        sys.exit(1)

    root = Path(__file__).resolve().parent.parent

    if not git_status_clean():
        print("Working tree is dirty. Commit or stash changes first.")
        sys.exit(1)

    tag = f"v{version}"
    print(f"Releasing {tag} ...")
    print()

    bump_workspace_version(root, version)

    subprocess.run(["git", "add", "Cargo.toml"], cwd=root, check=True)
    subprocess.run(["git", "commit", "-m", f"release: {tag}"], cwd=root, check=True)
    subprocess.run(["git", "tag", tag], cwd=root, check=True)

    print()
    print(f"Tag {tag} created.")
    print()
    print("Next:")
    print(f"  git push origin master {tag}")


if __name__ == "__main__":
    main()

"""Release script — bump version, commit, tag, push.

Usage:
  python scripts/release.py patch      # 0.1.2 → 0.1.3
  python scripts/release.py minor      # 0.1.3 → 0.2.0
  python scripts/release.py major      # 0.2.0 → 1.0.0
  python scripts/release.py 0.3.0      # Explicit version
"""

import re
import subprocess
import sys
from pathlib import Path


def read_version(root: Path) -> str:
    cargo_toml = root / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")
    m = re.search(r'^\[workspace\.package\]\s*[\s\S]*?^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if not m:
        print("error: version not found in [workspace.package]")
        sys.exit(1)
    return m.group(1)


def bump_version(current: str, target: str) -> str:
    parts = current.split(".")
    if len(parts) != 3:
        print(f"error: unexpected version format: {current}")
        sys.exit(1)
    major, minor, patch = int(parts[0]), int(parts[1]), int(parts[2])

    if target == "patch":
        patch += 1
    elif target == "minor":
        minor += 1
        patch = 0
    elif target == "major":
        major += 1
        minor = 0
        patch = 0
    else:
        return target

    return f"{major}.{minor}.{patch}"


def write_version(root: Path, new_version: str):
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
    cargo_toml.write_text(updated, encoding="utf-8")
    print(f"Updated workspace version to {new_version} in Cargo.toml")


def git_status_clean() -> bool:
    result = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True)
    return result.stdout.strip() == ""


def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/release.py <patch|minor|major|X.Y.Z>")
        sys.exit(1)

    arg = sys.argv[1]

    root = Path(__file__).resolve().parent.parent
    if not git_status_clean():
        print("Working tree is dirty. Commit or stash changes first.")
        sys.exit(1)

    current = read_version(root)
    new_version = bump_version(current, arg)
    tag = f"v{new_version}"

    print(f"Bumping {current} → {new_version}")
    print()

    write_version(root, new_version)
    subprocess.run(["cargo", "check", "--workspace"], cwd=root, check=True)
    subprocess.run(["git", "add", "Cargo.toml", "Cargo.lock"], cwd=root, check=True)
    subprocess.run(["git", "commit", "-m", f"release: {tag}"], cwd=root, check=True)
    subprocess.run(["git", "tag", tag], cwd=root, check=True)

    print(f"Tag {tag} created. Pushing ...")
    subprocess.run(["git", "push", "origin", "master", tag], cwd=root, check=True)
    print(f"Release {tag} pushed — GitHub Actions building 4 platforms.")


if __name__ == "__main__":
    main()

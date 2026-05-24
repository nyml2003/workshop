"""Smoke tests for workc CLI and infra.

Usage:
  python scripts/smoke_test.py                          # Full smoke test (CLI + infra)
  python scripts/smoke_test.py --cli-only               # CLI tests only
  python scripts/smoke_test.py --infra-only             # Infra tests only
  python scripts/smoke_test.py --binary path/to/exe     # Specify binary path
"""

import argparse
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile

# ── helpers ──────────────────────────────────────────────────────────────────

class SmokeTest:
    def __init__(self, binary: str):
        self.binary = os.path.abspath(binary)
        self.passed = 0
        self.failed = 0
        self.skipped = 0

    def _run(self, *args: str) -> subprocess.CompletedProcess:
        cmd = [self.binary, *args]
        return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")

    def test(self, name: str, *args: str,
             expect_fail: bool = False,
             skip: bool = False,
             check_out: str | None = None,
             check_code: bool = True):
        header = "CLI" if getattr(self, "_cli_section", False) else "INFRA"
        if skip:
            print(f"  SKIP : [{header}] {name}")
            self.skipped += 1
            return
        result = self._run(*args)
        ok = True

        if check_code:
            if expect_fail:
                if result.returncode == 0:
                    ok = False
            else:
                if result.returncode != 0:
                    ok = False

        if ok and check_out is not None:
            combined = result.stdout + result.stderr
            if isinstance(check_out, str):
                if check_out not in combined:
                    ok = False
            elif callable(check_out):
                if not check_out(combined):
                    ok = False

        if ok:
            print(f"  PASS : [{header}] {name}")
            self.passed += 1
        else:
            out = (result.stdout + result.stderr).strip()
            summary = out[:120].replace("\n", " ") if out else "(no output)"
            print(f"  FAIL : [{header}] {name}  [exit={result.returncode}] {summary}")
            self.failed += 1
        return result

    def section(self, title: str):
        print(f"\n  -- {title} --")

    def summary(self):
        total = self.passed + self.failed + self.skipped
        print()
        print(f"  {'=' * 46}")
        print(f"  TOTAL  : {total}")
        print(f"  PASSED : {self.passed}")
        print(f"  FAILED : {self.failed}")
        if self.skipped:
            print(f"  SKIPPED: {self.skipped}")
        print(f"  {'=' * 46}")
        return self.failed == 0


# ── CLI smoke tests ──────────────────────────────────────────────────────────

def run_cli_tests(t: SmokeTest, workspace: str):
    t._cli_section = True
    unique = random.randint(1000, 9999)
    slug = f"smoke{unique}"
    skill_name = f"test-skill-{unique}"
    repo_name = f"smoke-repo-{unique}"
    group_name = f"smoke-group-{unique}"

    t.section("task create")
    r = t.test("create", "task", "create",
               "--slug", slug, "--title", "Smoke Test", "--template", "default",
               "--description", "Smoke test task",
               "--source-brief", "smoke-source",
               "--tags", "demo,test",
               check_out=lambda o: "task-" in o)
    stdout = r.stdout + r.stderr
    m = re.search(r'task-\d{8}-[\w-]+', stdout)
    task_id = m.group(0) if m else None

    t.section("list")
    t.test("list current workspace", "list", check_out=slug)
    t.test("list --json", "--json", "list", check_out='"slug"')
    t.test("list --status active", "list", "--status", "active", check_out=slug)
    t.test("list --tag demo", "list", "--tag", "demo", check_out=slug)

    t.test("open without --editor (expect fail)", "open", slug, expect_fail=True)

    t.section("repo")
    t.test("repo add", "repo", "add", repo_name, "https://github.com/smoke/repo.git",
           "--tags", "rust,cli", "--description", "Smoke test repo", check_code=True)
    t.test("repo list", "repo", "list", check_out=repo_name)
    t.test("repo list --json", "--json", "repo", "list", check_out='"id"')

    t.section("repo-group")
    t.test("repo-group add", "repo-group", "add", group_name, repo_name,
           "--tags", "group-tag", "--description", "Smoke test group", check_code=True)
    t.test("repo-group list", "repo-group", "list", check_out=group_name)

    t.section("task repos")
    t.test("task repos set", "task", "repos", "set", slug,
           "--repos", repo_name, check_code=True)

    t.section("knowledge candidate")
    t.test("candidate create", "knowledge", "candidate", "create", slug, "cand1",
           "--title", "Test Knowledge", "--category", "test",
           "--tags", "k1,k2", "--source", "docs/readme", check_code=True)
    t.test("candidate list", "knowledge", "candidate", "list", slug, check_out="cand1")
    t.test("candidate show", "knowledge", "candidate", "show", slug, "cand1",
           check_out="Test Knowledge")
    t.test("candidate update-meta", "knowledge", "candidate", "update-meta",
           slug, "cand1", "--title", "Updated", "--category", "updated", check_code=True)
    t.test("knowledge promote", "knowledge", "promote", slug, "cand1", "k1", check_code=True)
    t.test("knowledge list (global)", "knowledge", "list", check_out="k1")
    t.test("knowledge show (global)", "knowledge", "show", "k1", check_out="Updated")
    t.test("knowledge update-meta (global)", "knowledge", "update-meta", "k1",
           "--title", "Final", "--category", "published", check_code=True)

    t.section("skill import (local)")
    skill_dir = os.path.join(workspace, "test-skill")
    os.makedirs(skill_dir, exist_ok=True)
    with open(os.path.join(skill_dir, "skill.toml"), "w", encoding="utf-8") as f:
        f.write(f'name = "{skill_name}"\nversion = "0.1.0"\ndescription = "Smoke test skill"\n')
    t.test("skill import local", "skill", "import", "local", "./test-skill",
           "--name", skill_name, "--version", "0.1.0", check_code=True)
    t.test("skill show", "skill", "show", skill_name, check_out=skill_name)
    t.test("skill mount", "skill", "mount", slug, skill_name, check_code=True)
    t.test("skill mounts", "skill", "mounts", task_id if task_id else slug,
           check_out=skill_name)
    t.test("skill versions", "skill", "versions", skill_name, check_code=True)

    t.section("task close")
    t.test("task close", "task", "close", task_id if task_id else slug, check_code=True)
    t.test("list --status closed", "list", "--status", "closed", check_out=slug)

    return {"repo_name": repo_name, "group_name": group_name}


# ── infra smoke tests ────────────────────────────────────────────────────────

def run_infra_tests(t: SmokeTest, workspace: str, repo_name: str, group_name: str):
    t._cli_section = False
    workc_home = os.path.join(os.path.expanduser("~"), ".workc")

    def _file(path: str, desc: str):
        if not os.path.isabs(path):
            path = os.path.join(workspace, path)
        ok = os.path.exists(path)
        short = path.replace("\\", "/").split("/")[-1] if len(path) > 50 else path
        label = "PASS" if ok else f"FAIL (missing: ...{short[-40:]})"
        print(f"  {label:<42} : {desc}")
        if ok:
            t.passed += 1
        else:
            t.failed += 1
        return ok

    def _content(path: str, fragment: str, desc: str):
        if not os.path.isabs(path):
            path = os.path.join(workspace, path)
        try:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
        except Exception:
            print(f"  FAIL (unreadable)                          : {desc}")
            t.failed += 1
            return
        ok = fragment in content
        label = "PASS" if ok else "FAIL (no match)"
        print(f"  {label:<42} : {desc}")
        if ok:
            t.passed += 1
        else:
            t.failed += 1

    t.section("workspace directory structure")
    _file(".workc.toml", ".workc.toml exists in CWD")
    _file("repos", "repos/ directory")
    _file("materials", "materials/ directory")
    _file("knowledge-candidates", "knowledge-candidates/ directory")
    _file("skills", "skills/ directory")
    _content(".workc.toml", "slug", ".workc.toml has slug field")
    _content(".workc.toml", "[paths]", ".workc.toml has [paths] section")

    t.section("skill mounts on-disk")
    _file("skills/mounts.toml", "mounts.toml in skills/")

    t.section("knowledge (global) on-disk")
    _file(os.path.join(workc_home, "knowledge/k1/meta.toml"), "global knowledge meta.toml")
    _content(os.path.join(workc_home, "knowledge/k1/meta.toml"), "Final", "knowledge has promoted content")

    t.section("global repo catalog")
    _file(os.path.join(workc_home, "repos/catalog.toml"), "repos/catalog.toml")
    _file(os.path.join(workc_home, "repos/groups.toml"), "repos/groups.toml")
    _content(os.path.join(workc_home, "repos/catalog.toml"), repo_name, "catalog contains test repo")
    _content(os.path.join(workc_home, "repos/groups.toml"), group_name, "groups contains test group")

    t.section("global skill registry")
    _file(os.path.join(workc_home, "skills/registry/sources.toml"), "sources.toml")
    _file(os.path.join(workc_home, "skills/registry/skills.toml"), "skills.toml")

    t.section("workspace registry")
    _file(os.path.join(workc_home, "workspaces.toml"), "workspaces.toml")

    t.section("data persistence")
    r = subprocess.run([t.binary, "list"], capture_output=True, text=True, encoding="utf-8")
    ok = r.returncode == 0 and "Smoke Test" in (r.stdout + r.stderr)
    label = "PASS" if ok else "FAIL"
    print(f"  {label:<42} : data persists across CLI invocations")
    if ok:
        t.passed += 1
    else:
        t.failed += 1


# ── main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="workc smoke test suite")
    parser.add_argument("--binary", default="target/release/workc-cli.exe",
                        help="Path to release binary")
    parser.add_argument("--cli-only", action="store_true", help="CLI tests only")
    parser.add_argument("--infra-only", action="store_true", help="Infra tests only")
    args = parser.parse_args()

    if not os.path.isfile(args.binary):
        print(f"Binary not found: {args.binary}", file=sys.stderr)
        sys.exit(1)

    t = SmokeTest(args.binary)
    workspace = tempfile.mkdtemp(prefix="workc-smoke-")
    prev_cwd = os.getcwd()

    try:
        os.chdir(workspace)
        print(f"workspace: {workspace}")
        print(f"binary   : {t.binary}")

        if not args.infra_only:
            ctx = run_cli_tests(t, workspace)

        if not args.cli_only:
            run_infra_tests(t, workspace, ctx["repo_name"], ctx["group_name"])

        ok = t.summary()
    finally:
        os.chdir(prev_cwd)
        shutil.rmtree(workspace, ignore_errors=True)

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

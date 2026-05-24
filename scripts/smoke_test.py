"""Smoke tests for workc CLI and infra.

Usage:
  python scripts/smoke_test.py                          # Full smoke test (CLI + infra)
  python scripts/smoke_test.py --cli-only               # CLI tests only
  python scripts/smoke_test.py --infra-only             # Infra tests only
  python scripts/smoke_test.py --binary path/to/exe     # Specify binary path
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import uuid

# ── helpers ──────────────────────────────────────────────────────────────────

class SmokeTest:
    def __init__(self, binary: str):
        self.binary = os.path.abspath(binary)
        self.passed = 0
        self.failed = 0
        self.skipped = 0

    def _run(self, *args: str, expect_fail: bool = False) -> subprocess.CompletedProcess:
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
        print(f"\n  ── {title} ──")

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

    t.section("task create")
    r = t.test("create basic", "task", "create",
               "--slug", "smoke1", "--title", "Smoke Test 1", "--template", "default",
               check_out=lambda o: "task-" in o)
    stdout = r.stdout + r.stderr
    m = re.search(r'task-\d{8}-[\w-]+', stdout)
    task_id = m.group(0) if m else None
    slug1 = "smoke1"

    t.test("create with tags", "task", "create",
           "--slug", "smoke2", "--title", "Smoke Test 2", "--template", "default",
           "--description", "A smoke test task",
           "--source-brief", "smoke-source",
           "--tags", "demo,test",
           check_out=lambda o: "task-" in o)

    t.section("list")
    t.test("list all", "list", check_out="smoke1")
    t.test("list --json", "--json", "list", check_out='"slug"')
    t.test("list --status active", "list", "--status", "active", check_out=slug1)
    t.test("list --tag demo", "list", "--tag", "demo", check_out="smoke2")
    t.test("list --limit 1", "list", "--limit", "1", check_code=True)

    # verify that --skills is still rejected (phase gate)
    t.test("create --skills (expect reject)", "task", "create",
           "--slug", "smoke4", "--title", "x", "--template", "default",
           "--skills", "test-skill",
           expect_fail=True)
    t.test("open without --editor (expect fail)", "open", "smoke1",
           expect_fail=True)

    t.section("repo")
    t.test("repo add", "repo", "add", "smoke-repo", "https://github.com/smoke/repo.git",
           "--tags", "rust,cli", "--description", "Smoke test repo", check_code=True)
    t.test("repo list", "repo", "list", check_out="smoke-repo")
    t.test("repo list --json", "--json", "repo", "list", check_out='"id"')

    t.section("repo-group")
    t.test("repo-group add", "repo-group", "add", "smoke-group", "smoke-repo",
           "--tags", "group-tag", "--description", "Smoke test group", check_code=True)
    t.test("repo-group list", "repo-group", "list", check_out="smoke-group")

    t.section("task repos")
    task_ref = task_id if task_id else "smoke1"
    t.test("task repos set", "task", "repos", "set", task_ref,
           "--repos", "smoke-repo", check_code=True)

    t.section("knowledge candidate")
    t.test("candidate create", "knowledge", "candidate", "create", task_ref, "cand1",
           "--title", "Test Knowledge", "--category", "test",
           "--tags", "k1,k2", "--source", "docs/readme", check_code=True)
    t.test("candidate list", "knowledge", "candidate", "list", task_ref,
           check_out="cand1")
    t.test("candidate show", "knowledge", "candidate", "show", task_ref, "cand1",
           check_out="Test Knowledge")
    t.test("candidate update-meta", "knowledge", "candidate", "update-meta",
           task_ref, "cand1", "--title", "Updated", "--category", "updated",
           check_code=True)
    t.test("knowledge promote", "knowledge", "promote", task_ref, "cand1", "k1",
           check_code=True)
    t.test("knowledge list (global)", "knowledge", "list", check_out="k1")
    t.test("knowledge show (global)", "knowledge", "show", "k1",
           check_out="Updated")
    t.test("knowledge update-meta (global)", "knowledge", "update-meta", "k1",
           "--title", "Final", "--category", "published", check_code=True)

    t.section("skill import (local)")
    skill_dir = os.path.join(workspace, "test-skill")
    os.makedirs(skill_dir, exist_ok=True)
    with open(os.path.join(skill_dir, "skill.toml"), "w", encoding="utf-8") as f:
        f.write('name = "test-skill"\nversion = "0.1.0"\ndescription = "Smoke test skill"\n')
    t.test("skill import local", "skill", "import", "local", "./test-skill",
           "--name", "test-skill", "--version", "0.1.0", check_code=True)
    t.test("skill show", "skill", "show", "test-skill", check_out="test-skill")
    t.test("skill mount", "skill", "mount", task_id, "test-skill", check_code=True)
    t.test("skill mounts", "skill", "mounts", task_id, check_out="test-skill")
    t.test("skill versions", "skill", "versions", "test-skill", check_code=True)

    t.section("task close")
    t.test("task close", "task", "close", task_ref, check_code=True)
    t.test("list --status closed", "list", "--status", "closed", check_out=slug1)

    t.section("task create --skills (skill exists)")
    t.test("create --skills succeed", "task", "create",
           "--slug", "smoke5", "--title", "With Skill", "--template", "default",
           "--skills", "test-skill", check_out="task-")


# ── infra smoke tests ────────────────────────────────────────────────────────

def run_infra_tests(t: SmokeTest, workspace: str):
    t._cli_section = False

    # Find task ID from filesystem
    tasks_dir = os.path.join(workspace, "tasks")
    real_task_id = None
    if os.path.isdir(tasks_dir):
        for entry in os.listdir(tasks_dir):
            if entry.startswith("task-"):
                real_task_id = entry
                break

    def _file(path: str, desc: str):
        full = os.path.join(workspace, path)
        ok = os.path.exists(full)
        label = "PASS" if ok else f"FAIL (missing: {path})"
        print(f"  {label:<42} : {desc}")
        if ok:
            t.passed += 1
        else:
            t.failed += 1
        return ok

    def _content(path: str, fragment: str, desc: str):
        full = os.path.join(workspace, path)
        try:
            with open(full, "r", encoding="utf-8") as f:
                content = f.read()
        except Exception:
            print(f"  FAIL (unreadable: {path:<40}) : {desc}")
            t.failed += 1
            return
        ok = fragment in content
        label = "PASS" if ok else "FAIL (no match)"
        print(f"  {label:<42} : {desc}")
        if ok:
            t.passed += 1
        else:
            t.failed += 1

    t.section("task directory structure")
    if real_task_id:
        _file(f"tasks/{real_task_id}/task.toml", f"tasks/<id>/task.toml")
        _file(f"tasks/{real_task_id}/repos", f"tasks/<id>/repos/")
        _file(f"tasks/{real_task_id}/materials", f"tasks/<id>/materials/")
        _content(f"tasks/{real_task_id}/task.toml", "[paths]", "task.toml has [paths] section")
        _content(f"tasks/{real_task_id}/task.toml", "slug", "task.toml has slug field")
        _content(f"tasks/{real_task_id}/task.toml", "title", "task.toml has title field")

    t.section("repo catalog on-disk")
    _file("repos/catalog.toml", "repos/catalog.toml")
    _file("repos/groups.toml", "repos/groups.toml")
    _content("repos/catalog.toml", "smoke-repo", "catalog contains smoke-repo")
    _content("repos/groups.toml", "smoke-group", "groups contains smoke-group")

    t.section("skill registry on-disk")
    _file("skills/registry/sources.toml", "skills/registry/sources.toml")
    _file("skills/registry/skills.toml", "skills/registry/skills.toml")

    t.section("skill mounts on-disk")
    if real_task_id:
        _file(f"tasks/{real_task_id}/.codex/skills/mounts.toml", "skill mounts.toml")

    t.section("knowledge (global) on-disk")
    _file("knowledge/k1/meta.toml", "global knowledge meta.toml")
    _content("knowledge/k1/meta.toml", "Final", "knowledge meta.toml has promoted content")

    t.section("data persistence")
    r = subprocess.run([t.binary, "list"], capture_output=True, text=True, encoding="utf-8")
    ok = r.returncode == 0 and "smoke1" in (r.stdout + r.stderr)
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
                        help="Path to release binary (default: target/release/workc-cli.exe)")
    parser.add_argument("--cli-only", action="store_true",
                        help="Run CLI tests only")
    parser.add_argument("--infra-only", action="store_true",
                        help="Run infra tests only")
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
            run_cli_tests(t, workspace)

        if not args.cli_only:
            run_infra_tests(t, workspace)

        ok = t.summary()
    finally:
        os.chdir(prev_cwd)
        shutil.rmtree(workspace, ignore_errors=True)

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

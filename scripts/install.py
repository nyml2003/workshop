"""workc installer — one-line setup, no cargo required.

Usage (online):
  python3 -c "$(curl -fsSL https://raw.githubusercontent.com/nyml2003/workshop/main/scripts/install.py)"

Usage (offline, from release zip):
  unzip workc-macos-arm64.zip && python3 install.py

All steps are idempotent — safe to re-run.
Uses pre-built binaries from GitHub Releases (no cargo/build needed).
"""

import os
import platform
import random
import shutil
import subprocess
import sys
import urllib.request
import zipfile

REPO = "nyml2003/workshop"
WORKC_HOME = os.path.join(os.path.expanduser("~"), ".workc")
BIN_DIR = os.path.join(WORKC_HOME, "bin")

PLATFORM_MAP = {
    ("Darwin",  "arm64"): "workc-macos-arm64.zip",
    ("Darwin",  "x86_64"): "workc-macos-x64.zip",
    ("Linux",   "x86_64"): "workc-linux-x64.zip",
    ("Windows", "AMD64"): "workc-windows-x64.zip",
    ("Windows", "x86_64"): "workc-windows-x64.zip",
}


def _bin_name() -> str:
    return "workc.exe" if platform.system() == "Windows" else "workc"


def _find_local_binary() -> str | None:
    """Check if workc binary is next to this script (release zip layout)."""
    candidates = [
        os.path.join(os.path.dirname(os.path.abspath(__file__)), _bin_name()),
        os.path.join(os.getcwd(), _bin_name()),
    ]
    for p in candidates:
        if os.path.isfile(p):
            return p
    return None


def _download_binary() -> str:
    """Download release zip, extract, return binary path."""
    os_name = platform.system()
    arch = platform.machine()
    key = (os_name, arch)
    if key not in PLATFORM_MAP:
        print(f"Unsupported platform: {os_name}/{arch}", file=sys.stderr)
        print("Available assets:", file=sys.stderr)
        for v in PLATFORM_MAP.values():
            print(f"  {v}", file=sys.stderr)
        sys.exit(1)

    zip_name = PLATFORM_MAP[key]
    url = f"https://github.com/{REPO}/releases/latest/download/{zip_name}"
    print(f"Downloading {url} ...")
    tmp = os.environ.get("TMPDIR", os.environ.get("TEMP", "/tmp"))
    tmp_zip = os.path.join(tmp, f"workc-{random.randint(1000,9999)}.zip")
    urllib.request.urlretrieve(url, tmp_zip)

    extract_dir = os.path.join(tmp, f"workc-extract-{random.randint(1000,9999)}")
    with zipfile.ZipFile(tmp_zip, "r") as zf:
        zf.extractall(extract_dir)
    os.unlink(tmp_zip)

    for root, _, files in os.walk(extract_dir):
        for f in files:
            if f == _bin_name():
                return os.path.join(root, f)

    print("Binary not found in downloaded zip", file=sys.stderr)
    sys.exit(1)


def _install_binary(binary_path: str):
    os.makedirs(BIN_DIR, exist_ok=True)
    dest = os.path.join(BIN_DIR, _bin_name())
    shutil.copy2(binary_path, dest)
    if platform.system() != "Windows":
        os.chmod(dest, 0o755)
    print(f"Installed binary to {dest}")


def _add_to_path():
    bin_entry = BIN_DIR.replace("\\", "/")
    if platform.system() == "Windows":
        subprocess.run(
            ["setx", "PATH", f"{BIN_DIR};%PATH%"],
            capture_output=True,
            text=True,
        )
        print(f"Added {BIN_DIR} to user PATH (restart terminal to apply)")
        return

    home = os.path.expanduser("~")
    profiles = []
    for candidate in [".zshrc", ".bashrc", ".bash_profile", ".profile"]:
        path = os.path.join(home, candidate)
        if os.path.isfile(path):
            profiles.append(path)
    if not profiles:
        profiles.append(os.path.join(home, ".zshrc"))

    for profile in profiles:
        line = f'export PATH="{bin_entry}:$PATH"\n'
        try:
            with open(profile, "r") as f:
                content = f.read()
        except FileNotFoundError:
            content = ""
        if bin_entry in content:
            print(f"PATH already configured in {profile}")
            continue
        with open(profile, "a") as f:
            f.write(f"\n# workc\n{line}")
        print(f"Added PATH entry to {profile}")

    print(f"Restart your terminal or run: export PATH=\"{bin_entry}:$PATH\"")


def _create_skeleton():
    dirs = [
        WORKC_HOME,
        BIN_DIR,
        os.path.join(WORKC_HOME, "repos"),
        os.path.join(WORKC_HOME, "skills", "registry"),
        os.path.join(WORKC_HOME, "skills", "cache"),
        os.path.join(WORKC_HOME, "knowledge"),
    ]
    for d in dirs:
        os.makedirs(d, exist_ok=True)


def _write_config():
    config_path = os.path.join(WORKC_HOME, "config.toml")
    if os.path.isfile(config_path):
        print(f"Config already exists at {config_path}, skipping.")
        return

    if not sys.stdin.isatty():
        print("No terminal — skipping knowledge remote (set it later in ~/.workc/config.toml)")
        return

    print()
    remote = input("Knowledge git remote (leave empty to skip): ").strip()
    if not remote:
        print("Skipping knowledge remote — you can set it later in ~/.workc/config.toml")
        return

    content = f'[knowledge]\nremote = "{remote}"\n'
    with open(config_path, "w") as f:
        f.write(content)
    print(f"Wrote {config_path}")


def _run_init():
    init_py = os.path.join(os.path.dirname(os.path.abspath(__file__)), "init.py")
    if os.path.isfile(init_py):
        subprocess.run([sys.executable, init_py], check=False)
    else:
        print("init.py not found, skipping knowledge clone.")


def main():
    print("workc installer")
    print("===============")
    print()

    binary = _find_local_binary()
    if binary:
        print(f"Found local binary: {binary}")
        print("  (offline mode — using binary from release zip)")
    else:
        print("No local binary found — downloading from GitHub Releases ...")
        binary = _download_binary()

    _install_binary(binary)
    _add_to_path()
    _create_skeleton()
    _write_config()
    _run_init()

    print()
    print("Installation complete!")
    print(f"  Binary: {BIN_DIR}")
    print(f"  Config: {WORKC_HOME}")
    print()
    print("Next: restart your terminal, then run:")
    print("  workc task create --slug my-task --title 'My Task' --template default")


if __name__ == "__main__":
    main()

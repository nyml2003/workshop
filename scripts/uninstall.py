"""workc uninstaller — removes ~/.workc/ and cleans up PATH.

Usage:
  python3 uninstall.py
"""

import os
import platform
import shutil
import sys

WORKC_HOME = os.path.join(os.path.expanduser("~"), ".workc")
BIN_DIR = os.path.join(WORKC_HOME, "bin")


def _remove_path_entries():
    bin_entry = BIN_DIR.replace("\\", "/")

    if platform.system() == "Windows":
        print(f"Windows: manually remove {BIN_DIR} from your PATH.")
        print("  Search 'edit environment variables' in Start Menu.")
        return

    home = os.path.expanduser("~")
    profiles = [os.path.join(home, p) for p in
                [".zshrc", ".bashrc", ".bash_profile", ".profile"]
                if os.path.isfile(os.path.join(home, p))]

    for profile in profiles:
        with open(profile, "r") as f:
            lines = f.readlines()

        new_lines = []
        in_workc_block = False
        for line in lines:
            if line.strip() == "# workc":
                in_workc_block = True
                continue
            if in_workc_block:
                if bin_entry in line:
                    in_workc_block = False
                    continue
                in_workc_block = False
            new_lines.append(line)

        # Also clean up any standalone PATH lines with workc
        final_lines = [l for l in new_lines if bin_entry not in l]

        with open(profile, "w") as f:
            f.writelines(final_lines)
        print(f"Cleaned PATH from {profile}")


def _remove_workc_home():
    if not os.path.isdir(WORKC_HOME):
        print(f"{WORKC_HOME} not found — nothing to remove.")
        return
    shutil.rmtree(WORKC_HOME)
    print(f"Removed {WORKC_HOME}")


def main():
    args = sys.argv[1:]
    force = "--yes" in args or "-y" in args

    if not force and sys.stdin.isatty():
        print("This will remove ~/.workc/ and clean up PATH entries.")
        resp = input("Continue? [y/N] ").strip().lower()
        if resp not in ("y", "yes"):
            print("Cancelled.")
            return

    _remove_path_entries()
    _remove_workc_home()
    print()
    print("Uninstall complete.")
    print("Restart your terminal for PATH changes to take effect.")

    _remove_path_entries()
    _remove_workc_home()
    print()
    print("Uninstall complete.")
    print("Restart your terminal for PATH changes to take effect.")


if __name__ == "__main__":
    main()

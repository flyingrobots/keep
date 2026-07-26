#!/usr/bin/env python3
"""Check every GitHub Actions workflow with the pinned actionlint tool."""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys

EXPECTED_VERSION = "1.7.12"


def find_actionlint() -> str:
    """Return the actionlint path or report a precise setup failure."""
    executable = shutil.which("actionlint")
    if executable is None:
        raise RuntimeError("actionlint is unavailable; install version 1.7.12")
    return executable


def verify_version(executable: str) -> None:
    """Refuse an actionlint version outside the reviewed tool boundary."""
    completed = subprocess.run(
        [executable, "-version"],
        check=False,
        capture_output=True,
        text=True,
    )
    first_line = completed.stdout.splitlines()[:1]
    observed = first_line[0] if first_line else "<missing>"
    if completed.returncode != 0 or observed != EXPECTED_VERSION:
        raise RuntimeError(
            f"actionlint version mismatch: "
            f"expected {EXPECTED_VERSION!r}, observed {observed!r}"
        )


def admit_workflow_path(path: str) -> bool:
    """Admit one existing regular workflow without following links."""
    try:
        mode = os.lstat(path).st_mode
    except FileNotFoundError:
        return False
    except OSError as error:
        raise RuntimeError(
            f"cannot inspect workflow source {path!r}: {error}"
        ) from error
    if not stat.S_ISREG(mode):
        raise RuntimeError(f"workflow source is not a regular file: {path!r}")
    return True


def workflow_paths() -> list[str]:
    """Return Git-admitted workflow paths in deterministic order."""
    completed = subprocess.run(
        [
            "git",
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-per-directory=.gitignore",
            "--",
            ".github/workflows/*.yml",
            ".github/workflows/*.yaml",
        ],
        check=False,
        stdout=subprocess.PIPE,
    )
    if completed.returncode != 0:
        raise RuntimeError("git ls-files failed while selecting workflows")
    paths = []
    for raw_path in completed.stdout.split(b"\0"):
        if raw_path:
            path = os.fsdecode(raw_path)
            if admit_workflow_path(path):
                paths.append(path)
    paths.sort()
    if not paths:
        raise RuntimeError("the GitHub Actions workflow corpus is empty")
    return paths


def main() -> int:
    """Run actionlint against the deterministic workflow input boundary."""
    try:
        executable = find_actionlint()
        verify_version(executable)
        paths = workflow_paths()
    except RuntimeError as error:
        print(f"Workflow check refused: {error}", file=sys.stderr)
        return 1

    completed = subprocess.run(
        [executable, "-shellcheck=", "-pyflakes=", *paths],
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Check the Git-admitted Markdown corpus and its internal links."""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys

EXPECTED_LINTER_VERSION = "markdownlint-cli2 v0.23.2 (markdownlint v0.41.1)"
EXPECTED_LINK_CHECKER_VERSION = "lychee 0.21.0"


def find_linter() -> str:
    """Return the Markdown linter path or report a precise setup failure."""
    executable = shutil.which("markdownlint-cli2")
    if executable is None:
        raise RuntimeError(
            "markdownlint-cli2 is unavailable; install version 0.23.2"
        )
    return executable


def find_link_checker() -> str:
    """Return the link checker path or report a precise setup failure."""
    executable = shutil.which("lychee")
    if executable is None:
        raise RuntimeError("lychee is unavailable; install version 0.21.0")
    return executable


def verify_linter_version(executable: str) -> None:
    """Refuse a Markdown linter version outside the reviewed tool boundary."""
    completed = subprocess.run(
        [executable, "--no-globs", "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    first_line = completed.stdout.splitlines()[:1]
    observed = first_line[0] if first_line else "<missing>"
    if completed.returncode != 0 or observed != EXPECTED_LINTER_VERSION:
        raise RuntimeError(
            f"markdownlint-cli2 version mismatch: "
            f"expected {EXPECTED_LINTER_VERSION!r}, observed {observed!r}"
        )


def verify_link_checker_version(executable: str) -> None:
    """Refuse a link checker version outside the reviewed tool boundary."""
    completed = subprocess.run(
        [executable, "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    first_line = completed.stdout.splitlines()[:1]
    observed = first_line[0] if first_line else "<missing>"
    if completed.returncode != 0 or observed != EXPECTED_LINK_CHECKER_VERSION:
        raise RuntimeError(
            f"lychee version mismatch: "
            f"expected {EXPECTED_LINK_CHECKER_VERSION!r}, "
            f"observed {observed!r}"
        )


def admit_source_path(path: str) -> bool:
    """Admit one existing regular file without following links."""
    try:
        mode = os.lstat(path).st_mode
    except FileNotFoundError:
        return False
    except OSError as error:
        raise RuntimeError(
            f"cannot inspect Markdown source {path!r}: {error}"
        ) from error
    if not stat.S_ISREG(mode):
        raise RuntimeError(f"Markdown source is not a regular file: {path!r}")
    return True


def source_markdown() -> list[str]:
    """Return tracked and nonignored new Markdown in deterministic order."""
    completed = subprocess.run(
        [
            "git",
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-per-directory=.gitignore",
            "--",
            "*.md",
        ],
        check=False,
        stdout=subprocess.PIPE,
    )
    if completed.returncode != 0:
        raise RuntimeError("git ls-files failed while selecting Markdown")
    paths = []
    for raw_path in completed.stdout.split(b"\0"):
        if raw_path:
            path = os.fsdecode(raw_path)
            if admit_source_path(path):
                paths.append(path)
    paths.sort()
    if not paths:
        raise RuntimeError("the source Markdown corpus is empty")
    return paths


def main() -> int:
    """Run deterministic Markdown and internal-link checks."""
    try:
        linter = find_linter()
        link_checker = find_link_checker()
        verify_linter_version(linter)
        verify_link_checker_version(link_checker)
        paths = source_markdown()
    except RuntimeError as error:
        print(f"Markdown check refused: {error}", file=sys.stderr)
        return 1

    lint_result = subprocess.run(
        [linter, "--no-globs", "--", *paths],
        check=False,
    )
    link_result = subprocess.run(
        [
            link_checker,
            "--offline",
            "--include-fragments",
            "--no-progress",
            "--format",
            "detailed",
            "--",
            *paths,
        ],
        check=False,
    )
    return lint_result.returncode or link_result.returncode


if __name__ == "__main__":
    sys.exit(main())

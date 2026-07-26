#!/usr/bin/env python3
"""Check the Git-admitted Markdown corpus with the pinned lint tool."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys

EXPECTED_VERSION = "markdownlint-cli2 v0.19.1 (markdownlint v0.39.0)"


def find_linter() -> str:
    """Return the Markdown linter path or report a precise setup failure."""
    executable = shutil.which("markdownlint-cli2")
    if executable is None:
        raise RuntimeError(
            "markdownlint-cli2 is unavailable; install version 0.19.1"
        )
    return executable


def verify_version(executable: str) -> None:
    """Refuse a Markdown linter version outside the reviewed tool boundary."""
    completed = subprocess.run(
        [executable, "--no-globs", "--version"],
        check=False,
        capture_output=True,
        text=True,
    )
    first_line = completed.stdout.splitlines()[:1]
    observed = first_line[0] if first_line else "<missing>"
    if completed.returncode != 0 or observed != EXPECTED_VERSION:
        raise RuntimeError(
            f"markdownlint-cli2 version mismatch: "
            f"expected {EXPECTED_VERSION!r}, observed {observed!r}"
        )


def source_markdown() -> list[str]:
    """Return tracked and nonignored new Markdown in deterministic order."""
    completed = subprocess.run(
        [
            "git",
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.md",
        ],
        check=False,
        stdout=subprocess.PIPE,
    )
    if completed.returncode != 0:
        raise RuntimeError("git ls-files failed while selecting Markdown")
    paths = sorted(
        os.fsdecode(path)
        for path in completed.stdout.split(b"\0")
        if path
    )
    if not paths:
        raise RuntimeError("the source Markdown corpus is empty")
    return paths


def main() -> int:
    """Run Markdownlint against the deterministic source input boundary."""
    try:
        executable = find_linter()
        verify_version(executable)
        paths = source_markdown()
    except RuntimeError as error:
        print(f"Markdown check refused: {error}", file=sys.stderr)
        return 1

    completed = subprocess.run(
        [executable, "--no-globs", "--", *paths],
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main())

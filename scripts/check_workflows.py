#!/usr/bin/env python3
"""Check every GitHub Actions workflow with the pinned actionlint tool."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

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


def workflow_paths() -> list[str]:
    """Return every YAML workflow path in deterministic order."""
    workflow_dir = Path(".github/workflows")
    paths = sorted(
        str(path)
        for pattern in ("*.yml", "*.yaml")
        for path in workflow_dir.glob(pattern)
    )
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

    completed = subprocess.run([executable, *paths], check=False)
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main())

"""Repository laws for bounded, auditable source modules."""

from __future__ import annotations

import os
import subprocess
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).parent.parent
SOURCE_SUFFIXES = frozenset({".py", ".rs", ".sh"})
SOURCE_MODULE_HARD_LIMIT_LINES = 500


def tracked_source_modules() -> tuple[Path, ...]:
    """Return tracked source modules in deterministic repository order."""
    completed = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
    )
    relative_paths = (
        Path(os.fsdecode(raw_path))
        for raw_path in completed.stdout.split(b"\0")
        if raw_path
    )
    return tuple(
        REPOSITORY_ROOT / relative_path
        for relative_path in relative_paths
        if relative_path.suffix in SOURCE_SUFFIXES
    )


class SourceModuleSizeLaws(unittest.TestCase):
    """Tracked source modules stay within Keep's hard audit bound."""

    def test_no_source_module_exceeds_hard_line_limit(self) -> None:
        violations = {}
        for module in tracked_source_modules():
            line_count = len(module.read_bytes().splitlines())
            if line_count > SOURCE_MODULE_HARD_LIMIT_LINES:
                relative_path = module.relative_to(REPOSITORY_ROOT)
                violations[relative_path.as_posix()] = line_count

        self.assertEqual(
            violations,
            {},
            "tracked source modules exceed the 500-line hard maximum",
        )


if __name__ == "__main__":
    unittest.main()

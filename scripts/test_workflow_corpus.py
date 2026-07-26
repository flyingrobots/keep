"""Laws for the repository-owned GitHub Actions source boundary."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from check_workflows import workflow_paths


class WorkflowCorpusLaws(unittest.TestCase):
    """The selected workflow corpus contains only admitted source files."""

    def setUp(self) -> None:
        original_directory = Path.cwd()
        self.addCleanup(os.chdir, original_directory)
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        self.root = Path(temporary_directory.name)
        os.chdir(self.root)
        self.run_git("init", "--quiet")

    def run_git(self, *arguments: str) -> None:
        """Run one Git setup command inside the isolated repository."""
        subprocess.run(
            ["git", *arguments],
            check=True,
            capture_output=True,
        )

    def write(self, path: str, content: str) -> None:
        """Create one regular UTF-8 fixture file."""
        destination = self.root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content, encoding="utf-8")

    def test_repository_ignored_workflow_cannot_enter_the_corpus(self) -> None:
        self.write(".gitignore", "/.github/workflows/generated.yml\n")
        self.write(".github/workflows/tracked.yml", "name: Tracked\n")
        self.write(".github/workflows/new.yml", "name: New\n")
        self.write(".github/workflows/generated.yml", "name: Generated\n")
        self.run_git(
            "add",
            ".gitignore",
            ".github/workflows/tracked.yml",
        )

        self.assertEqual(
            workflow_paths(),
            [
                ".github/workflows/new.yml",
                ".github/workflows/tracked.yml",
            ],
        )

    def test_source_paths_have_one_canonical_lexical_order(self) -> None:
        self.write(".github/workflows/zulu.yml", "name: Zulu\n")
        self.write(".github/workflows/alpha.yaml", "name: Alpha\n")
        self.run_git(
            "add",
            ".github/workflows/zulu.yml",
            ".github/workflows/alpha.yaml",
        )

        self.assertEqual(
            workflow_paths(),
            [
                ".github/workflows/alpha.yaml",
                ".github/workflows/zulu.yml",
            ],
        )

    def test_deleted_tracked_workflow_is_not_forwarded(self) -> None:
        self.write(".github/workflows/deleted.yml", "name: Deleted\n")
        self.write(".github/workflows/remaining.yml", "name: Remaining\n")
        self.run_git(
            "add",
            ".github/workflows/deleted.yml",
            ".github/workflows/remaining.yml",
        )
        (self.root / ".github/workflows/deleted.yml").unlink()

        self.assertEqual(
            workflow_paths(),
            [".github/workflows/remaining.yml"],
        )

    def test_user_global_ignores_cannot_change_the_corpus(self) -> None:
        self.write(".github/workflows/tracked.yml", "name: Tracked\n")
        self.write(".github/workflows/new.yml", "name: New\n")
        self.write("global-ignore", "*.yml\n")
        self.run_git("add", ".github/workflows/tracked.yml")
        global_config = self.root / "global.gitconfig"
        subprocess.run(
            [
                "git",
                "config",
                "--file",
                str(global_config),
                "core.excludesFile",
                str(self.root / "global-ignore"),
            ],
            check=True,
            capture_output=True,
        )

        with patch.dict(
            os.environ,
            {
                "GIT_CONFIG_GLOBAL": str(global_config),
                "GIT_CONFIG_NOSYSTEM": "1",
            },
        ):
            self.assertEqual(
                workflow_paths(),
                [
                    ".github/workflows/new.yml",
                    ".github/workflows/tracked.yml",
                ],
            )

    def test_symlinked_workflow_is_refused(self) -> None:
        self.write("generated.yml", "name: Generated\n")
        workflow_dir = self.root / ".github/workflows"
        workflow_dir.mkdir(parents=True)
        (workflow_dir / "linked.yml").symlink_to("../../generated.yml")

        with self.assertRaisesRegex(RuntimeError, "not a regular file"):
            workflow_paths()

    def test_fifo_workflow_cannot_enter_the_corpus(self) -> None:
        self.write(".github/workflows/regular.yml", "name: Regular\n")
        workflow_dir = self.root / ".github/workflows"
        os.mkfifo(workflow_dir / "blocking.yml")

        self.assertEqual(
            workflow_paths(),
            [".github/workflows/regular.yml"],
        )


if __name__ == "__main__":
    unittest.main()

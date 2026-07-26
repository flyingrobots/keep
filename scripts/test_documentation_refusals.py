"""Precise refusal laws for malformed documentation and workflows."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

import check_markdown
import check_workflows


class IsolatedRepositoryTestCase(unittest.TestCase):
    """Own one temporary Git repository for a malformed source fixture."""

    def setUp(self) -> None:
        original_directory = Path.cwd()
        self.addCleanup(os.chdir, original_directory)
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        self.root = Path(temporary_directory.name)
        os.chdir(self.root)
        subprocess.run(
            ["git", "init", "--quiet"],
            check=True,
            capture_output=True,
        )

    def write(self, path: str, content: str) -> None:
        """Create one regular UTF-8 fixture file."""
        destination = self.root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content, encoding="utf-8")


class IntegrityRefusalLaws(IsolatedRepositoryTestCase):
    """Malformed inputs produce the expected checker failure class."""

    def test_broken_internal_fragment_is_refused(self) -> None:
        self.write(
            "source.md",
            "# Source\n\n[Missing](target.md#missing-heading)\n",
        )
        self.write("target.md", "# Present heading\n")
        subprocess.run(
            ["git", "add", "source.md", "target.md"],
            check=True,
            capture_output=True,
        )
        linter = check_markdown.find_linter()
        link_checker = check_markdown.find_link_checker()
        check_markdown.verify_linter_version(linter)
        check_markdown.verify_link_checker_version(link_checker)
        paths = check_markdown.source_markdown()

        lint = subprocess.run(
            [linter, "--no-globs", "--", *paths],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(lint.returncode, 0, lint.stderr)
        links = subprocess.run(
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
            capture_output=True,
            text=True,
        )
        self.assertEqual(links.returncode, 2)
        self.assertIn(
            "Cannot find fragment",
            f"{links.stdout}\n{links.stderr}",
        )

    def test_invalid_workflow_is_refused(self) -> None:
        self.write(
            ".github/workflows/invalid.yml",
            "name: Invalid\non: [push\n",
        )
        actionlint = check_workflows.find_actionlint()
        check_workflows.verify_version(actionlint)
        paths = check_workflows.workflow_paths()

        completed = subprocess.run(
            [actionlint, "-shellcheck=", "-pyflakes=", *paths],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 1)
        self.assertIn(
            "could not parse as YAML",
            f"{completed.stdout}\n{completed.stderr}",
        )


if __name__ == "__main__":
    unittest.main()

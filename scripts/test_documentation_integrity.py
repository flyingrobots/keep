"""Negative laws for the required documentation integrity gates."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import check_markdown
import check_workflows

REPOSITORY_ROOT = Path(__file__).parent.parent
LAW_COMMAND = "python3 -m unittest discover -s scripts -p 'test_*.py' -v"


class IsolatedRepositoryTestCase(unittest.TestCase):
    """Own one temporary Git repository for a negative gate fixture."""

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
    """Malformed documentation and workflows never enter required CI."""

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

        self.assertNotEqual(check_markdown.main(), 0)

    def test_invalid_workflow_is_refused(self) -> None:
        self.write(
            ".github/workflows/invalid.yml",
            "name: Invalid\non: [push\n",
        )

        self.assertNotEqual(check_workflows.main(), 0)


class ToolVersionLaws(unittest.TestCase):
    """Every required checker refuses an unexpected executable version."""

    def test_wrong_markdownlint_version_is_refused(self) -> None:
        with patch.object(
            check_markdown.subprocess,
            "run",
            return_value=SimpleNamespace(
                returncode=0,
                stdout="markdownlint-cli2 v999.0.0\n",
            ),
        ):
            with self.assertRaisesRegex(RuntimeError, "version mismatch"):
                check_markdown.verify_linter_version("markdownlint-cli2")

    def test_wrong_lychee_version_is_refused(self) -> None:
        with patch.object(
            check_markdown.subprocess,
            "run",
            return_value=SimpleNamespace(
                returncode=0,
                stdout="lychee 999.0.0\n",
            ),
        ):
            with self.assertRaisesRegex(RuntimeError, "version mismatch"):
                check_markdown.verify_link_checker_version("lychee")

    def test_wrong_actionlint_version_is_refused(self) -> None:
        with patch.object(
            check_workflows.subprocess,
            "run",
            return_value=SimpleNamespace(
                returncode=0,
                stdout="999.0.0\n",
            ),
        ):
            with self.assertRaisesRegex(RuntimeError, "version mismatch"):
                check_workflows.verify_version("actionlint")


class WorkflowContractLaws(unittest.TestCase):
    """Required CI executes the repository's negative integrity laws."""

    def test_documentation_job_runs_negative_integrity_laws(self) -> None:
        workflow = (
            REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"
        ).read_text(encoding="utf-8")
        self.assertIn(LAW_COMMAND, workflow)

    def test_actionlint_disables_unadmitted_auxiliary_linters(self) -> None:
        completed = SimpleNamespace(returncode=0)
        with (
            patch.object(
                check_workflows,
                "find_actionlint",
                return_value="actionlint",
            ),
            patch.object(check_workflows, "verify_version"),
            patch.object(
                check_workflows,
                "workflow_paths",
                return_value=[".github/workflows/ci.yml"],
            ),
            patch.object(
                check_workflows.subprocess,
                "run",
                return_value=completed,
            ) as run,
        ):
            self.assertEqual(check_workflows.main(), 0)

        run.assert_called_once_with(
            [
                "actionlint",
                "-shellcheck=",
                "-pyflakes=",
                ".github/workflows/ci.yml",
            ],
            check=False,
        )


class ToolInstallerLaws(unittest.TestCase):
    """The Markdown tool graph is fully locked before network installation."""

    def test_known_parser_denial_of_service_versions_are_refused(self) -> None:
        lock_path = (
            REPOSITORY_ROOT
            / "scripts"
            / "documentation-tools"
            / "package-lock.json"
        )
        packages = json.loads(lock_path.read_text(encoding="utf-8"))[
            "packages"
        ]
        self.assertEqual(
            packages["node_modules/js-yaml"]["version"],
            "5.2.2",
        )
        self.assertEqual(
            packages["node_modules/markdown-it"]["version"],
            "14.3.0",
        )

    def test_markdown_dependency_graph_is_lockfile_admitted(self) -> None:
        tool_directory = (
            REPOSITORY_ROOT / "scripts" / "documentation-tools"
        )
        lock_path = tool_directory / "package-lock.json"
        self.assertTrue(lock_path.is_file())
        lock = json.loads(lock_path.read_text(encoding="utf-8"))
        self.assertEqual(lock["lockfileVersion"], 3)
        self.assertEqual(
            lock["packages"][""]["dependencies"]["markdownlint-cli2"],
            "0.23.1",
        )
        for path, package in lock["packages"].items():
            if path:
                self.assertIn("resolved", package, path)
                self.assertIn("integrity", package, path)

        installer = (
            REPOSITORY_ROOT / "scripts" / "install_documentation_tools.sh"
        ).read_text(encoding="utf-8")
        self.assertIn("npm ci", installer)
        self.assertIn("package-lock.json", installer)
        self.assertNotIn("npm install \\", installer)


if __name__ == "__main__":
    unittest.main()

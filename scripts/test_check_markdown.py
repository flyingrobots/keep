"""Laws for the repository-owned Markdown source boundary."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from check_markdown import source_markdown

REPOSITORY_ROOT = Path(__file__).parent.parent


class MarkdownCorpusLaws(unittest.TestCase):
    """The selected Markdown corpus depends only on admitted source files."""

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

    def test_repository_ignored_markdown_cannot_enter_the_corpus(self) -> None:
        self.write(".gitignore", "/target/\n")
        self.write("tracked.md", "# Tracked\n")
        self.write("new.md", "# New\n")
        self.write("target/generated.md", "# Generated\n")
        self.run_git("add", ".gitignore", "tracked.md")

        self.assertEqual(source_markdown(), ["new.md", "tracked.md"])

    def test_source_paths_have_canonical_lexical_order(self) -> None:
        self.write("zulu.md", "# Zulu\n")
        self.write("alpha.md", "# Alpha\n")
        self.run_git("add", "zulu.md", "alpha.md")

        self.assertEqual(source_markdown(), ["alpha.md", "zulu.md"])

    def test_deleted_tracked_markdown_is_not_forwarded(self) -> None:
        self.write("deleted.md", "# Deleted\n")
        self.write("remaining.md", "# Remaining\n")
        self.run_git("add", "deleted.md", "remaining.md")
        (self.root / "deleted.md").unlink()

        self.assertEqual(source_markdown(), ["remaining.md"])

    def test_user_global_ignores_cannot_change_the_corpus(self) -> None:
        self.write("tracked.md", "# Tracked\n")
        self.write("new.md", "# New\n")
        self.write("global-ignore", "*.md\n")
        self.run_git("add", "tracked.md")
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
            self.assertEqual(source_markdown(), ["new.md", "tracked.md"])

    def test_symlinked_markdown_is_refused(self) -> None:
        self.write("target/generated.md", "# Generated\n")
        (self.root / "linked.md").symlink_to("target/generated.md")

        with self.assertRaisesRegex(RuntimeError, "not a regular file"):
            source_markdown()

    def test_fifo_markdown_is_refused(self) -> None:
        self.write("blocking.md", "# Initially regular\n")
        self.run_git("add", "blocking.md")
        (self.root / "blocking.md").unlink()
        os.mkfifo(self.root / "blocking.md")

        with self.assertRaisesRegex(RuntimeError, "not a regular file"):
            source_markdown()


class DocumentationCommandLaws(unittest.TestCase):
    """Contributor commands inspect changes that have not been committed."""

    def test_whitespace_checks_cover_the_index_and_working_tree(self) -> None:
        for relative_path in (
            "CONTRIBUTING.md",
            "docs/Documentation Standards.md",
        ):
            source = (REPOSITORY_ROOT / relative_path).read_text(
                encoding="utf-8"
            )
            self.assertNotIn(
                'git diff --check "$(git hash-object -t tree /dev/null)" HEAD',
                source,
            )
            self.assertIn("git diff --check\n", source)
            self.assertIn("git diff --cached --check\n", source)


if __name__ == "__main__":
    unittest.main()

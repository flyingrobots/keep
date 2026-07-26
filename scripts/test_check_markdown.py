"""Laws for the repository-owned Markdown source boundary."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from check_markdown import source_markdown


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


if __name__ == "__main__":
    unittest.main()

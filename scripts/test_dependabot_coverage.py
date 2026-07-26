"""Laws for complete, uniform Dependabot manifest coverage."""

from __future__ import annotations

import os
import subprocess
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).parent.parent
CONFIG_PATH = REPOSITORY_ROOT / ".github" / "dependabot.yml"
UPDATE_MARKER = "  - package-ecosystem: "
MANIFEST_ECOSYSTEMS = {
    "Cargo.toml": "cargo",
    "package.json": "npm",
}
STATIC_SCOPES = {("github-actions", "/")}


def update_blocks(raw: str) -> list[str]:
    """Return each Dependabot update block in source order."""
    lines = raw.splitlines()
    starts = [
        index
        for index, line in enumerate(lines)
        if line.startswith(UPDATE_MARKER)
    ]
    return [
        "\n".join(lines[start:end])
        for start, end in zip(starts, [*starts[1:], len(lines)], strict=True)
    ]


def unquote(raw: str) -> str:
    """Remove one matching YAML scalar quote pair."""
    if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in {"'", '"'}:
        return raw[1:-1]
    return raw


def configured_scopes(blocks: list[str]) -> list[tuple[str, str]]:
    """Return every configured ecosystem and manifest directory pair."""
    scopes: list[tuple[str, str]] = []
    for block in blocks:
        lines = block.splitlines()
        ecosystem = unquote(lines[0].removeprefix(UPDATE_MARKER))
        for index, line in enumerate(lines):
            if line.startswith("    directory: "):
                scopes.append(
                    (ecosystem, unquote(line.removeprefix("    directory: ")))
                )
            if line == "    directories:":
                for directory_line in lines[index + 1 :]:
                    if not directory_line.startswith("      - "):
                        break
                    scopes.append(
                        (
                            ecosystem,
                            unquote(directory_line.removeprefix("      - ")),
                        )
                    )
    return scopes


def tracked_manifest_scopes() -> set[tuple[str, str]]:
    """Return dependency scopes implied by tracked first-party manifests."""
    completed = subprocess.run(
        [
            "git",
            "ls-files",
            "-z",
            "--",
            ":(glob)**/Cargo.toml",
            ":(glob)**/package.json",
        ],
        cwd=REPOSITORY_ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    scopes = set()
    for raw_path in completed.stdout.split(b"\0"):
        if not raw_path:
            continue
        path = Path(os.fsdecode(raw_path))
        ecosystem = MANIFEST_ECOSYSTEMS[path.name]
        directory = (
            "/"
            if path.parent == Path(".")
            else f"/{path.parent.as_posix()}"
        )
        scopes.add((ecosystem, directory))
    return scopes.union(STATIC_SCOPES)


class DependabotCoverageLaws(unittest.TestCase):
    """Every first-party manifest receives one uniform update policy."""

    def setUp(self) -> None:
        self.raw = CONFIG_PATH.read_text(encoding="utf-8")
        self.blocks = update_blocks(self.raw)

    def test_every_dependency_manifest_has_an_update_scope(self) -> None:
        observed = set(configured_scopes(self.blocks))
        self.assertEqual(tracked_manifest_scopes().difference(observed), set())

    def test_update_scopes_are_unique(self) -> None:
        scopes = configured_scopes(self.blocks)
        self.assertEqual(len(scopes), len(set(scopes)))

    def test_every_update_uses_the_maintenance_policy(self) -> None:
        self.assertTrue(self.raw.startswith("version: 2\nupdates:\n"))
        self.assertTrue(self.blocks)
        for block in self.blocks:
            self.assertIn("    schedule:\n      interval: weekly", block)
            self.assertIn("    open-pull-requests-limit: 5", block)
            self.assertIn("    labels:\n      - dependencies", block)


if __name__ == "__main__":
    unittest.main()

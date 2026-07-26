"""Laws for the bounded GitHub fuzz campaign boundary."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).parent.parent
WORKFLOW_DIRECTORY = REPOSITORY_ROOT / ".github" / "workflows"
SCHEDULED_WORKFLOW = WORKFLOW_DIRECTORY / "fuzz-scheduled.yml"
ACTION_REFERENCE = re.compile(r"^\s*uses:\s*[^@\s]+@([^\s]+)", re.MULTILINE)
COMMIT_SHA = re.compile(r"[0-9a-f]{40}")


def step(workflow: str, name: str) -> str:
    """Return one workflow step block by its exact reviewed name."""
    marker = f"      - name: {name}\n"
    start = workflow.index(marker)
    end = workflow.find("\n      - name:", start + len(marker))
    if end == -1:
        return workflow[start:]
    return workflow[start:end]


class WorkflowLaws(unittest.TestCase):
    """The scheduled boundary remains pinned, bounded, and fail-closed."""

    def setUp(self) -> None:
        self.workflow = SCHEDULED_WORKFLOW.read_text(encoding="utf-8")

    def test_every_third_party_action_uses_an_immutable_commit(self) -> None:
        workflow_paths = sorted(WORKFLOW_DIRECTORY.glob("*.yml"))
        references = [
            reference
            for path in workflow_paths
            for reference in ACTION_REFERENCE.findall(
                path.read_text(encoding="utf-8")
            )
        ]
        self.assertTrue(references)
        for reference in references:
            self.assertIsNotNone(COMMIT_SHA.fullmatch(reference))

    def test_scheduled_campaign_is_restricted_to_main(self) -> None:
        self.assertIn("  schedule:\n", self.workflow)
        self.assertIn("  workflow_dispatch:\n", self.workflow)
        self.assertIn(
            "    if: github.ref == 'refs/heads/main'\n",
            self.workflow,
        )

    def test_refused_corpus_restore_removes_a_root_link(self) -> None:
        discard = step(self.workflow, "Discard a refused corpus restore")
        self.assertIn("test -L fuzz/corpus", discard)
        self.assertIn("find -P fuzz/corpus -depth -delete", discard)

    def test_only_successfully_minimized_corpora_are_retained(self) -> None:
        for name in (
            "Save non-authoritative evolving corpus",
            "Retain minimized corpus evidence",
        ):
            retained = step(self.workflow, name)
            self.assertIn(
                "steps.campaign.outcome == 'success'",
                retained,
            )
            self.assertIn(
                "steps.minimize.outcome == 'success'",
                retained,
            )
            self.assertIn(
                "steps.retained_corpus.outcome == 'success'",
                retained,
            )


if __name__ == "__main__":
    unittest.main()

"""Laws for the deterministic fuzz campaign controller."""

from __future__ import annotations

import io
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import campaign_execution
import campaign_policy
import check_corpus


class CampaignPolicyLaws(unittest.TestCase):
    """The policy admits only complete, bounded, exact configuration."""

    def setUp(self) -> None:
        self.raw = campaign_policy.POLICY_PATH.read_text(encoding="utf-8")

    def test_repository_policy_is_complete_and_bounded(self) -> None:
        policy = campaign_policy.parse_policy(self.raw)
        self.assertGreater(
            policy.scheduled_seconds_per_target,
            policy.smoke_seconds_per_target,
        )
        self.assertLessEqual(policy.max_input_bytes, 1_048_576)

    def test_smoke_profile_preserves_reviewed_runtime_bounds(self) -> None:
        policy = campaign_policy.parse_policy(self.raw)
        self.assertEqual(policy.smoke_seconds_per_target, 15)
        self.assertEqual(policy.input_timeout_seconds, 5)
        self.assertEqual(policy.max_input_bytes, 1_048_576)
        self.assertEqual(policy.rss_limit_mb, 1_024)

    def test_duplicate_policy_key_is_refused(self) -> None:
        version_line = next(
            line
            for line in self.raw.splitlines()
            if line.startswith("CARGO_FUZZ_VERSION=")
        )
        duplicated = f"{self.raw}\n{version_line}\n"
        with self.assertRaisesRegex(
            campaign_policy.PolicyError,
            "unknown, duplicate, or empty",
        ):
            campaign_policy.parse_policy(duplicated)

    def test_out_of_bound_policy_value_is_refused(self) -> None:
        moved = "\n".join(
            "FUZZ_RSS_LIMIT_MB=999999"
            if line.startswith("FUZZ_RSS_LIMIT_MB=")
            else line
            for line in self.raw.splitlines()
        )
        with self.assertRaisesRegex(
            campaign_policy.PolicyError,
            "outside",
        ):
            campaign_policy.parse_policy(moved)


class CampaignExecutionLaws(unittest.TestCase):
    """Every registered target runs and every failure remains visible."""

    def setUp(self) -> None:
        self.policy = campaign_policy.load_policy()
        self.targets = sorted(
            path.stem
            for path in campaign_execution.TARGET_DIRECTORY.glob("*.rs")
        )
        self.assertTrue(self.targets)

    @patch("campaign_execution.subprocess.run")
    def test_target_listing_is_sorted_and_exact(self, run: unittest.mock.Mock) -> None:
        run.return_value = SimpleNamespace(
            returncode=0,
            stdout="\n".join(reversed(self.targets)) + "\n",
            stderr="",
        )
        observed = campaign_execution.registered_targets(
            "cargo",
            self.policy,
        )
        self.assertEqual(observed, self.targets)

    @patch("campaign_execution.subprocess.run")
    def test_every_target_runs_after_an_earlier_failure(
        self,
        run: unittest.mock.Mock,
    ) -> None:
        run.side_effect = [SimpleNamespace(returncode=1)] + [
            SimpleNamespace(returncode=0)
            for _target in self.targets[1:]
        ]
        with patch(
            "campaign_execution.sys.stderr",
            new_callable=io.StringIO,
        ):
            status = campaign_execution.run_targets(
                "cargo",
                self.policy,
                "smoke",
                self.targets,
            )
        self.assertEqual(status, 1)
        self.assertEqual(run.call_count, len(self.targets))

    @patch("campaign_execution.subprocess.run")
    def test_swallowed_cmin_failure_is_refused(
        self,
        run: unittest.mock.Mock,
    ) -> None:
        run.return_value = SimpleNamespace(
            returncode=0,
            stdout="Failed to minimize corpus: signal 6\n",
            stderr="",
        )
        with patch(
            "campaign_execution.sys.stdout",
            new_callable=io.StringIO,
        ):
            self.assertFalse(
                campaign_execution.minimize_target(
                    "cargo",
                    self.policy,
                    self.targets[0],
                )
            )


class CorpusAdmissionLaws(unittest.TestCase):
    """Retained corpus state remains regular, recognized, and bounded."""

    def setUp(self) -> None:
        self.policy = campaign_policy.load_policy()

    def test_missing_corpus_is_normal_absence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "missing"
            stats = check_corpus.audit_corpus(root, self.policy)
        self.assertEqual(stats, check_corpus.CorpusStats(files=0, bytes=0))

    def test_unknown_target_directory_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "unknown").mkdir()
            with self.assertRaisesRegex(
                check_corpus.CorpusError,
                "unexpected corpus target",
            ):
                check_corpus.audit_corpus(root, self.policy)

    def test_oversized_input_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "blob_hasher"
            target.mkdir()
            oversized = target / "oversized"
            oversized.write_bytes(
                bytes(self.policy.max_input_bytes + 1)
            )
            with self.assertRaisesRegex(
                check_corpus.CorpusError,
                "exceeds the input bound",
            ):
                check_corpus.audit_corpus(root, self.policy)

    def test_linked_input_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "blob_hasher"
            target.mkdir()
            source = root / "source"
            source.write_bytes(b"seed")
            (target / "linked").symlink_to(source)
            with self.assertRaisesRegex(
                check_corpus.CorpusError,
                "not a regular file",
            ):
                check_corpus.audit_corpus(root, self.policy)


if __name__ == "__main__":
    unittest.main()

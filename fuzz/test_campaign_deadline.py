"""Laws for externally enforced fuzz campaign deadlines."""

from __future__ import annotations

import io
import subprocess
import unittest
from unittest.mock import patch

import campaign_execution
import campaign_policy


class CampaignDeadlineLaws(unittest.TestCase):
    """A child process cannot outlive its admitted operation budget."""

    @patch("campaign_execution.subprocess.run")
    def test_expired_cmin_deadline_is_refused(
        self,
        run: unittest.mock.Mock,
    ) -> None:
        policy = campaign_policy.load_policy()

        def expire(_command: list[str], **arguments: object) -> None:
            self.assertEqual(
                arguments.get("timeout"),
                policy.cmin_seconds_per_target,
            )
            raise subprocess.TimeoutExpired(
                cmd="cargo fuzz cmin",
                timeout=policy.cmin_seconds_per_target,
            )

        run.side_effect = expire
        with patch(
            "campaign_execution.sys.stderr",
            new_callable=io.StringIO,
        ) as stderr:
            completed = campaign_execution.minimize_target(
                "cargo",
                policy,
                "blob_hasher",
            )

        self.assertFalse(completed)
        self.assertIn("timed out", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()

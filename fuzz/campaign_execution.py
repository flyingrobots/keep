"""Own bounded Cargo execution for every registered fuzz target."""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
from pathlib import Path

from campaign_policy import CampaignPolicy

TARGET_DIRECTORY = Path(__file__).with_name("fuzz_targets")
TARGET_PATTERN = re.compile(r"[a-z][a-z0-9_]*")
CMIN_FAILURE_MARKER = "Failed to minimize corpus:"


def cargo_executable() -> str:
    """Return Cargo's path or report the missing execution boundary."""
    executable = shutil.which("cargo")
    if executable is None:
        raise RuntimeError("cargo is unavailable")
    return executable


def registered_targets(cargo: str, policy: CampaignPolicy) -> list[str]:
    """Return validated targets exactly matching the checked-in harnesses."""
    completed = subprocess.run(
        [cargo, f"+{policy.toolchain}", "fuzz", "list"],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        sys.stdout.write(completed.stdout)
        sys.stderr.write(completed.stderr)
        raise RuntimeError("cargo fuzz list failed")

    targets = [line.strip() for line in completed.stdout.splitlines()]
    if not targets or any(not target for target in targets):
        raise RuntimeError("cargo fuzz list returned no targets")
    if len(set(targets)) != len(targets):
        raise RuntimeError("cargo fuzz list returned duplicate targets")
    if any(TARGET_PATTERN.fullmatch(target) is None for target in targets):
        raise RuntimeError("cargo fuzz list returned a malformed target name")

    observed = sorted(targets)
    expected = sorted(path.stem for path in TARGET_DIRECTORY.glob("*.rs"))
    if observed != expected:
        raise RuntimeError(
            f"registered fuzz targets differ from harnesses: "
            f"expected {expected!r}, observed {observed!r}"
        )
    return observed


def common_fuzzer_arguments(policy: CampaignPolicy) -> list[str]:
    """Return the shared bounded libFuzzer arguments."""
    return [
        f"-timeout={policy.input_timeout_seconds}",
        f"-max_len={policy.max_input_bytes}",
        f"-rss_limit_mb={policy.rss_limit_mb}",
        "-print_final_stats=1",
    ]


def build_targets(
    cargo: str,
    policy: CampaignPolicy,
    targets: list[str],
) -> int:
    """Build every target and aggregate all target-specific failures."""
    failures: list[str] = []
    for target in targets:
        completed = subprocess.run(
            [cargo, f"+{policy.toolchain}", "fuzz", "build", target],
            check=False,
        )
        if completed.returncode != 0:
            failures.append(target)
    return report_failures("build", failures)


def run_targets(
    cargo: str,
    policy: CampaignPolicy,
    profile: str,
    targets: list[str],
) -> int:
    """Exercise every target even when an earlier target finds a failure."""
    arguments = [
        f"-max_total_time={policy.seconds_per_target(profile)}",
        *common_fuzzer_arguments(policy),
    ]
    failures: list[str] = []
    for target in targets:
        completed = subprocess.run(
            [
                cargo,
                f"+{policy.toolchain}",
                "fuzz",
                "run",
                target,
                "--",
                *arguments,
            ],
            check=False,
        )
        if completed.returncode != 0:
            failures.append(target)
    return report_failures("campaign", failures)


def minimize_target(
    cargo: str,
    policy: CampaignPolicy,
    target: str,
) -> bool:
    """Minimize one corpus and detect cargo-fuzz's swallowed failure marker."""
    completed = subprocess.run(
        [
            cargo,
            f"+{policy.toolchain}",
            "fuzz",
            "cmin",
            target,
            "--",
            f"-max_total_time={policy.cmin_seconds_per_target}",
            *common_fuzzer_arguments(policy),
        ],
        check=False,
        capture_output=True,
        text=True,
        errors="replace",
    )
    sys.stdout.write(completed.stdout)
    sys.stderr.write(completed.stderr)
    combined = completed.stdout + completed.stderr
    return completed.returncode == 0 and CMIN_FAILURE_MARKER not in combined


def minimize_targets(
    cargo: str,
    policy: CampaignPolicy,
    targets: list[str],
) -> int:
    """Minimize every corpus and aggregate all minimization failures."""
    failures = [
        target
        for target in targets
        if not minimize_target(cargo, policy, target)
    ]
    return report_failures("corpus minimization", failures)


def report_failures(operation: str, failures: list[str]) -> int:
    """Report the exact failed target set and return a process status."""
    if not failures:
        return 0
    print(
        f"fuzz {operation} failed for: {', '.join(failures)}",
        file=sys.stderr,
    )
    return 1

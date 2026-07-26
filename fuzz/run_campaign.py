#!/usr/bin/env python3
"""Run every registered fuzz target under a bounded campaign policy."""

from __future__ import annotations

import argparse
import sys

from campaign_execution import (
    build_targets,
    cargo_executable,
    minimize_targets,
    registered_targets,
    run_targets,
)
from campaign_policy import CampaignPolicy, PolicyError, load_policy


def print_environment(policy: CampaignPolicy, profile: str) -> int:
    """Emit validated GitHub environment assignments."""
    for key, value in sorted(policy.environment(profile).items()):
        print(f"{key}={value}")
    return 0


def print_description(policy: CampaignPolicy, profile: str) -> int:
    """Describe the admitted campaign without invoking Cargo."""
    for key, value in sorted(policy.environment(profile).items()):
        print(f"{key}: {value}")
    return 0


def parse_arguments() -> argparse.Namespace:
    """Parse the bounded campaign command and profile."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "command",
        choices=("build", "describe", "github-env", "minimize", "run"),
    )
    parser.add_argument(
        "--profile",
        choices=("scheduled", "smoke"),
        default="smoke",
    )
    return parser.parse_args()


def main() -> int:
    """Validate policy and execute the requested campaign operation."""
    arguments = parse_arguments()
    try:
        policy = load_policy()
        if arguments.command == "github-env":
            return print_environment(policy, arguments.profile)
        if arguments.command == "describe":
            return print_description(policy, arguments.profile)
        cargo = cargo_executable()
        targets = registered_targets(cargo, policy)
    except (PolicyError, RuntimeError) as error:
        print(f"fuzz campaign refused: {error}", file=sys.stderr)
        return 1

    if arguments.command == "build":
        return build_targets(cargo, policy, targets)
    if arguments.command == "minimize":
        return minimize_targets(cargo, policy, targets)
    return run_targets(cargo, policy, arguments.profile, targets)


if __name__ == "__main__":
    sys.exit(main())

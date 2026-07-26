"""Own and validate the bounded fuzz campaign policy."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

POLICY_PATH = Path(__file__).with_name("campaign.env")
EXPECTED_KEYS = {
    "CARGO_FUZZ_VERSION",
    "FUZZ_CMIN_SECONDS_PER_TARGET",
    "FUZZ_CORPUS_MAX_BYTES",
    "FUZZ_CORPUS_MAX_FILES",
    "FUZZ_CORPUS_RETENTION_DAYS",
    "FUZZ_INPUT_TIMEOUT_SECONDS",
    "FUZZ_MAX_INPUT_BYTES",
    "FUZZ_RSS_LIMIT_MB",
    "FUZZ_SCHEDULED_SECONDS_PER_TARGET",
    "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS",
    "FUZZ_SMOKE_FAILURE_RETENTION_DAYS",
    "FUZZ_SMOKE_SECONDS_PER_TARGET",
    "FUZZ_TOOLCHAIN",
}
SEMVER_PATTERN = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+")
TOOLCHAIN_PATTERN = re.compile(r"nightly-[0-9]{4}-[0-9]{2}-[0-9]{2}")


class PolicyError(ValueError):
    """The campaign policy is missing, malformed, or outside its bounds."""


@dataclass(frozen=True)
class CampaignPolicy:
    """Validated tool versions and campaign resource limits."""

    cargo_fuzz_version: str
    toolchain: str
    input_timeout_seconds: int
    max_input_bytes: int
    rss_limit_mb: int
    smoke_seconds_per_target: int
    scheduled_seconds_per_target: int
    cmin_seconds_per_target: int
    smoke_failure_retention_days: int
    scheduled_failure_retention_days: int
    corpus_retention_days: int
    corpus_max_files: int
    corpus_max_bytes: int

    def seconds_per_target(self, profile: str) -> int:
        """Return the exploration budget for a named campaign profile."""
        if profile == "smoke":
            return self.smoke_seconds_per_target
        if profile == "scheduled":
            return self.scheduled_seconds_per_target
        raise PolicyError(f"unknown fuzz campaign profile: {profile!r}")

    def environment(self, profile: str) -> dict[str, str]:
        """Return the exact environment exported to a workflow."""
        return {
            "CARGO_FUZZ_VERSION": self.cargo_fuzz_version,
            "FUZZ_CMIN_SECONDS_PER_TARGET": str(
                self.cmin_seconds_per_target
            ),
            "FUZZ_CORPUS_RETENTION_DAYS": str(
                self.corpus_retention_days
            ),
            "FUZZ_CORPUS_MAX_BYTES": str(self.corpus_max_bytes),
            "FUZZ_CORPUS_MAX_FILES": str(self.corpus_max_files),
            "FUZZ_INPUT_TIMEOUT_SECONDS": str(
                self.input_timeout_seconds
            ),
            "FUZZ_MAX_INPUT_BYTES": str(self.max_input_bytes),
            "FUZZ_RSS_LIMIT_MB": str(self.rss_limit_mb),
            "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS": str(
                self.scheduled_failure_retention_days
            ),
            "FUZZ_SECONDS_PER_TARGET": str(
                self.seconds_per_target(profile)
            ),
            "FUZZ_SMOKE_FAILURE_RETENTION_DAYS": str(
                self.smoke_failure_retention_days
            ),
            "FUZZ_TOOLCHAIN": self.toolchain,
        }


def parse_assignments(raw: str) -> dict[str, str]:
    """Parse a strict, substitution-free KEY=VALUE policy file."""
    assignments: dict[str, str] = {}
    for line_number, raw_line in enumerate(raw.splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.count("=") != 1:
            raise PolicyError(f"line {line_number} is not KEY=VALUE")
        key, value = line.split("=", maxsplit=1)
        if key not in EXPECTED_KEYS or key in assignments or not value:
            raise PolicyError(
                f"line {line_number} has an unknown, duplicate, or empty key"
            )
        if any(character.isspace() for character in value):
            raise PolicyError(f"line {line_number} contains whitespace")
        assignments[key] = value
    missing = EXPECTED_KEYS.difference(assignments)
    if missing:
        raise PolicyError(f"campaign policy is missing: {sorted(missing)}")
    return assignments


def bounded_integer(
    assignments: dict[str, str],
    key: str,
    minimum: int,
    maximum: int,
) -> int:
    """Parse one decimal policy value within an explicit inclusive bound."""
    raw = assignments[key]
    if not raw.isascii() or not raw.isdecimal():
        raise PolicyError(f"{key} is not an ASCII decimal integer")
    value = int(raw)
    if not minimum <= value <= maximum:
        raise PolicyError(f"{key} is outside [{minimum}, {maximum}]")
    return value


def parse_policy(raw: str) -> CampaignPolicy:
    """Admit a complete campaign policy after validating every field."""
    values = parse_assignments(raw)
    cargo_fuzz_version = values["CARGO_FUZZ_VERSION"]
    toolchain = values["FUZZ_TOOLCHAIN"]
    if SEMVER_PATTERN.fullmatch(cargo_fuzz_version) is None:
        raise PolicyError("CARGO_FUZZ_VERSION is not an exact version")
    if TOOLCHAIN_PATTERN.fullmatch(toolchain) is None:
        raise PolicyError("FUZZ_TOOLCHAIN is not a dated nightly")

    policy = CampaignPolicy(
        cargo_fuzz_version=cargo_fuzz_version,
        toolchain=toolchain,
        input_timeout_seconds=bounded_integer(
            values, "FUZZ_INPUT_TIMEOUT_SECONDS", 1, 60
        ),
        max_input_bytes=bounded_integer(
            values, "FUZZ_MAX_INPUT_BYTES", 1, 1_048_576
        ),
        rss_limit_mb=bounded_integer(
            values, "FUZZ_RSS_LIMIT_MB", 128, 8_192
        ),
        smoke_seconds_per_target=bounded_integer(
            values, "FUZZ_SMOKE_SECONDS_PER_TARGET", 1, 60
        ),
        scheduled_seconds_per_target=bounded_integer(
            values, "FUZZ_SCHEDULED_SECONDS_PER_TARGET", 60, 3_600
        ),
        cmin_seconds_per_target=bounded_integer(
            values, "FUZZ_CMIN_SECONDS_PER_TARGET", 1, 600
        ),
        smoke_failure_retention_days=bounded_integer(
            values, "FUZZ_SMOKE_FAILURE_RETENTION_DAYS", 1, 90
        ),
        scheduled_failure_retention_days=bounded_integer(
            values, "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS", 1, 90
        ),
        corpus_retention_days=bounded_integer(
            values, "FUZZ_CORPUS_RETENTION_DAYS", 1, 90
        ),
        corpus_max_files=bounded_integer(
            values, "FUZZ_CORPUS_MAX_FILES", 1, 100_000
        ),
        corpus_max_bytes=bounded_integer(
            values, "FUZZ_CORPUS_MAX_BYTES", 1, 1_073_741_824
        ),
    )
    if policy.scheduled_seconds_per_target <= policy.smoke_seconds_per_target:
        raise PolicyError("scheduled fuzzing must exceed the smoke budget")
    if policy.corpus_max_bytes < policy.max_input_bytes:
        raise PolicyError("corpus bytes cannot be smaller than one input")
    return policy


def load_policy() -> CampaignPolicy:
    """Read and admit the repository-owned campaign policy."""
    try:
        raw = POLICY_PATH.read_text(encoding="utf-8")
    except OSError as error:
        raise PolicyError(f"cannot read {POLICY_PATH}: {error}") from error
    return parse_policy(raw)

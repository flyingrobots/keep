"""Independent Golden File Worldline capability posture contract."""

from __future__ import annotations

import re

from corpus_protocol import (
    ROOT,
    U16_MAX,
    U64_MAX,
    decimal,
    fail,
    rows,
    unique,
)

CAPABILITY_COLUMNS = [
    "capability",
    "posture",
    "first_milestone",
    "owning_issues",
    "claim",
]
CAPABILITY_PATTERN = re.compile(
    r"keep(?:\.[a-z0-9-]+)+/v[1-9][0-9]*\Z"
)
CAPABILITY_CONTRACTS = {
    "keep.identity.canonical/v1": ("required", 1, (2, 6)),
    "keep.identity.partition-invariant/v1": ("required", 1, (6,)),
    "keep.model.exact-immutable-map/v1": ("required", 1, (5,)),
    "keep.cdc.profile.canonical/v1": ("required", 2, (7,)),
    "keep.content.exact-public-read/v1": (
        "declared-future",
        2,
        (13,),
    ),
    "keep.cdc.nearby-state-reuse/v1": (
        "declared-future",
        2,
        (7, 8, 13),
    ),
    "keep.ingest.bounded-stream/v1": ("declared-future", 2, (13,)),
    "keep.range.minimal-overlap/v1": ("declared-future", 2, (11,)),
    "keep.segment.verified-read/v1": (
        "declared-future",
        3,
        (14, 15),
    ),
    "keep.restart.lawful-recovery/v1": (
        "declared-future",
        3,
        (17,),
    ),
    "keep.retention.both-states/v1": (
        "declared-future",
        4,
        (18, 19),
    ),
    "keep.verification.precise-refusal/v1": (
        "declared-future",
        4,
        (20,),
    ),
    "keep.compaction.identity-stable/v1": (
        "declared-future",
        4,
        (21,),
    ),
    "keep.echo.identity-agreement/v1": (
        "declared-future",
        5,
        (22, 23),
    ),
    "keep.graft.golden-worldline/v1": (
        "declared-future",
        5,
        (24,),
    ),
    "keep.git-cas.import/v1": ("declared-future", 5, (25,)),
}


def issue_numbers(value: str, capability: str) -> tuple[int, ...]:
    parts = value.split(",")
    issues = tuple(
        decimal(part, f"{capability} issue", U64_MAX)
        for part in parts
    )
    if (
        not issues
        or any(issue == 0 for issue in issues)
        or tuple(sorted(set(issues))) != issues
    ):
        fail(
            f"{capability}: owning issues are empty, duplicate, "
            "or unordered"
        )
    return issues


def check_capabilities() -> None:
    capability_rows = rows(
        ROOT / "capabilities.tsv",
        "# keep.golden-file-worldline.capabilities/v1",
        CAPABILITY_COLUMNS,
    )
    seen: set[str] = set()
    for row in capability_rows:
        capability = row["capability"]
        if CAPABILITY_PATTERN.fullmatch(capability) is None:
            fail(
                "capabilities.tsv: invalid coordinate "
                f"{capability!r}"
            )
        unique(capability, seen, "capabilities.tsv")
        milestone = row["first_milestone"]
        if not milestone.startswith("M"):
            fail(f"{capability}: malformed milestone")
        observed = (
            row["posture"],
            decimal(
                milestone[1:],
                f"{capability} milestone",
                U16_MAX,
            ),
            issue_numbers(row["owning_issues"], capability),
        )
        expected = CAPABILITY_CONTRACTS.get(capability)
        if expected is None or observed != expected:
            fail(f"{capability}: capability contract moved")
        if not row["claim"] or row["claim"] != row["claim"].strip():
            fail(f"{capability}: claim is empty or noncanonical")
    if seen != set(CAPABILITY_CONTRACTS):
        fail("capabilities.tsv: required v1 capability set moved")

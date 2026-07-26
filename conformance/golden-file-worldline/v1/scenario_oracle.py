"""Independent Golden File Worldline mutation and scenario laws."""

from __future__ import annotations

from corpus_protocol import (
    MAX_MUTATION_VALUE_BYTES,
    MAX_SOURCE_BYTES,
    ROOT,
    case_name,
    decimal,
    decoded_hex,
    fail,
    rows,
    unique,
)
from identity_oracle import (
    ALGORITHM,
    BINARY_LENGTH,
    ID_MAGIC,
    VERSION,
    IdentityFixture,
    digest,
    expected_text,
)

STEP_COLUMNS = [
    "step",
    "operation",
    "input_case",
    "identity_case",
    "expected_outcome",
]
MUTATION_COLUMNS = [
    "case",
    "target_kind",
    "target_case",
    "operation",
    "offset",
    "value_hex",
    "expected_outcome",
]
STEP_OUTCOMES = {
    "identify": "keep.identity.identified",
    "admit-exact": "keep.content.admitted",
    "read-exact": "keep.content.exact",
    "verify-claimed-content": "keep.content.mismatch",
    "read-absent": "keep.content.absent",
}
REQUIRED_STEP_OPERATIONS = [
    "identify",
    "admit-exact",
    "read-exact",
    "identify",
    "admit-exact",
    "read-exact",
    "read-exact",
    "verify-claimed-content",
    "read-exact",
    "read-absent",
]
REQUIRED_MUTATION_OPERATIONS = {
    ("content", "xor-byte"),
    ("content", "truncate"),
    ("content", "append"),
    ("identity-binary", "xor-byte"),
    ("identity-binary", "truncate"),
    ("identity-binary", "append"),
    ("identity-binary", "set-u8"),
    ("identity-binary", "set-u16-be"),
}


def mutate(
    target: bytes,
    operation: str,
    offset: int,
    value_field: str,
    name: str,
) -> bytes:
    changed = bytearray(target)
    if operation == "truncate":
        if value_field != "-" or offset >= len(changed):
            fail(f"{name}: invalid truncation")
        del changed[offset:]
    elif operation == "append":
        value = decoded_hex(
            value_field,
            f"{name} value",
            MAX_MUTATION_VALUE_BYTES,
        )
        if offset != len(changed):
            fail(f"{name}: append offset does not equal target length")
        changed.extend(value)
    else:
        width = {
            "xor-byte": 1,
            "set-u8": 1,
            "set-u16-be": 2,
        }.get(operation)
        if width is None:
            fail(f"{name}: unknown mutation operation {operation!r}")
        value = decoded_hex(value_field, f"{name} value", width)
        if len(value) != width or offset + width > len(changed):
            fail(f"{name}: mutation width or offset is invalid")
        if operation == "xor-byte":
            changed[offset] ^= value[0]
        else:
            changed[offset : offset + width] = value
    result = bytes(changed)
    if (
        result == target
        or len(result) > MAX_SOURCE_BYTES + MAX_MUTATION_VALUE_BYTES
    ):
        fail(f"{name}: mutation is a no-op or exceeds its bound")
    return result


def binary_outcomes(encoded: bytes) -> set[str]:
    if len(encoded) < BINARY_LENGTH:
        return {"keep.identity.truncated"}
    if len(encoded) > BINARY_LENGTH:
        return {"keep.identity.trailing_data"}
    if encoded[:16] != ID_MAGIC:
        return {"keep.identity.invalid_magic"}
    if int.from_bytes(encoded[16:18], "big") != VERSION:
        return {"keep.identity.unsupported_version"}
    if encoded[18] != ALGORITHM:
        return {"keep.identity.unsupported_algorithm"}
    return {"keep.identity.different_supported_identity"}


def check_mutations(
    fixtures: dict[str, IdentityFixture],
) -> None:
    mutation_rows = rows(
        ROOT / "mutations.tsv",
        "# keep.golden-file-worldline.mutations/v1",
        MUTATION_COLUMNS,
    )
    seen: set[str] = set()
    covered: set[tuple[str, str]] = set()
    for row in mutation_rows:
        name = case_name(row["case"], "mutations.tsv")
        unique(name, seen, "mutations.tsv")
        target_kind = row["target_kind"]
        if target_kind not in {"content", "identity-binary"}:
            fail(f"{name}: unknown mutation target kind")
        fixture = fixtures.get(row["target_case"])
        if fixture is None:
            fail(f"{name}: mutation target case is absent")
        target = (
            fixture.content
            if target_kind == "content"
            else fixture.canonical_binary
        )
        offset = decimal(row["offset"], f"{name} offset", len(target))
        changed = mutate(
            target,
            row["operation"],
            offset,
            row["value_hex"],
            name,
        )
        if target_kind == "content":
            observed = expected_text(len(changed), digest(changed))
            if (
                row["expected_outcome"] != "keep.content.mismatch"
                or observed == fixture.canonical_text
            ):
                fail(
                    f"{name}: content mutation does not prove mismatch"
                )
        elif row["expected_outcome"] not in binary_outcomes(changed):
            fail(f"{name}: binary mutation outcome is incorrect")
        covered.add((target_kind, row["operation"]))
    if not REQUIRED_MUTATION_OPERATIONS.issubset(covered):
        fail("mutations.tsv: required v1 mutation coverage is absent")


def check_steps(fixtures: dict[str, IdentityFixture]) -> None:
    step_rows = rows(
        ROOT / "steps.tsv",
        "# keep.golden-file-worldline.steps/v1",
        STEP_COLUMNS,
    )
    admitted: set[bytes] = set()
    operations: list[str] = []
    for expected_number, row in enumerate(step_rows, start=1):
        number = decimal(
            row["step"],
            "scenario step",
            len(step_rows),
        )
        if number != expected_number:
            fail(
                "steps.tsv: step numbers are not canonical and contiguous"
            )
        operation = row["operation"]
        outcome = STEP_OUTCOMES.get(operation)
        if outcome is None or row["expected_outcome"] != outcome:
            fail(f"steps.tsv:{number}: operation outcome is invalid")
        identity = fixtures.get(row["identity_case"])
        if identity is None:
            fail(f"steps.tsv:{number}: identity case is absent")
        if operation == "read-absent":
            if (
                row["input_case"] != "-"
                or identity.canonical_binary in admitted
            ):
                fail(f"steps.tsv:{number}: absent read is not absent")
        else:
            source = fixtures.get(row["input_case"])
            if source is None:
                fail(f"steps.tsv:{number}: input case is absent")
            same_identity = (
                source.canonical_binary == identity.canonical_binary
            )
            if operation == "verify-claimed-content" and same_identity:
                fail(f"steps.tsv:{number}: mismatch uses matching content")
            if (
                operation != "verify-claimed-content"
                and not same_identity
            ):
                fail(
                    f"steps.tsv:{number}: exact operation substitutes "
                    "identity"
                )
            if operation == "admit-exact":
                admitted.add(identity.canonical_binary)
            if (
                operation == "read-exact"
                and identity.canonical_binary not in admitted
            ):
                fail(
                    f"steps.tsv:{number}: exact read precedes admission"
                )
        operations.append(operation)
    if operations != REQUIRED_STEP_OPERATIONS:
        fail(
            "steps.tsv: ordered Golden File Worldline v1 operations moved"
        )

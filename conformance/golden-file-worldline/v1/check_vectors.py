#!/usr/bin/env python3
"""Validate Golden File Worldline v1 without importing Keep."""

from __future__ import annotations

from dataclasses import dataclass
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
DATA_MAGIC = b"KEEP:BLOB:DATA\0\0"
ID_MAGIC = b"KEEP:BLOB:ID\0\0\0\0"
VERSION = 1
ALGORITHM = 1
BINARY_LENGTH = 59
MAX_TEXT_BYTES = 109
MAX_SOURCE_BYTES = 1_048_576
MAX_TOTAL_BYTES = 1_048_911
MAX_TABLE_BYTES = 1_048_576
MAX_INVALID_TEXT_BYTES = 4_096
MAX_MUTATION_VALUE_BYTES = 64
U16_MAX = (1 << 16) - 1
U64_MAX = (1 << 64) - 1
CASE_PATTERN = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*\Z")
CAPABILITY_PATTERN = re.compile(r"keep(?:\.[a-z0-9-]+)+/v[1-9][0-9]*\Z")
LOWER_HEX = frozenset("0123456789abcdef")

IDENTITY_COLUMNS = [
    "case", "source_kind", "source_parameter", "repetitions",
    "logical_length", "canonical_text", "canonical_binary_hex",
]
STEP_COLUMNS = ["step", "operation", "input_case", "identity_case", "expected_outcome"]
INVALID_TEXT_COLUMNS = ["case", "input_hex", "expected_outcome"]
MUTATION_COLUMNS = [
    "case", "target_kind", "target_case", "operation",
    "offset", "value_hex", "expected_outcome",
]
CAPABILITY_COLUMNS = ["capability", "posture", "first_milestone", "owning_issues", "claim"]

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
REQUIRED_TEXT_OUTCOMES = {
    "keep.identity.input_too_long",
    "keep.identity.malformed_structure",
    "keep.identity.trailing_data",
    "keep.identity.invalid_scheme",
    "keep.identity.invalid_kind",
    "keep.identity.malformed_version",
    "keep.identity.unsupported_version",
    "keep.identity.unsupported_algorithm",
    "keep.identity.noncanonical_length",
    "keep.identity.length_overflow",
    "keep.identity.invalid_digest_length",
    "keep.identity.noncanonical_digest_case",
    "keep.identity.invalid_digest_alphabet",
}
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
CAPABILITY_CONTRACTS = {
    "keep.identity.canonical/v1": ("required", 1, (2, 6)),
    "keep.identity.partition-invariant/v1": ("required", 1, (6,)),
    "keep.model.exact-immutable-map/v1": ("required", 1, (5,)),
    "keep.content.exact-public-read/v1": ("declared-future", 2, (13,)),
    "keep.cdc.nearby-state-reuse/v1": ("declared-future", 2, (7, 8, 13)),
    "keep.ingest.bounded-stream/v1": ("declared-future", 2, (13,)),
    "keep.range.minimal-overlap/v1": ("declared-future", 2, (11,)),
    "keep.segment.verified-read/v1": ("declared-future", 3, (14, 15)),
    "keep.restart.lawful-recovery/v1": ("declared-future", 3, (17,)),
    "keep.retention.both-states/v1": ("declared-future", 4, (18, 19)),
    "keep.verification.precise-refusal/v1": ("declared-future", 4, (20,)),
    "keep.compaction.identity-stable/v1": ("declared-future", 4, (21,)),
    "keep.echo.identity-agreement/v1": ("declared-future", 5, (22, 23)),
    "keep.graft.golden-worldline/v1": ("declared-future", 5, (24,)),
    "keep.git-cas.import/v1": ("declared-future", 5, (25,)),
}

@dataclass(frozen=True)
class IdentityFixture:
    content: bytes
    canonical_text: str
    canonical_binary: bytes

def fail(message: str) -> None:
    raise SystemExit(f"golden corpus check failed: {message}")

def bounded_file_bytes(path: pathlib.Path, maximum: int, label: str) -> bytes:
    with path.open("rb") as source:
        content = source.read(maximum + 1)
    if len(content) > maximum:
        fail(f"{label}: file exceeds {maximum} bytes")
    return content

def protocol_lines(path: pathlib.Path) -> list[str]:
    size = path.stat().st_size
    if size == 0 or size > MAX_TABLE_BYTES:
        fail(f"{path.name}: protocol size {size} is outside its bound")
    raw = bounded_file_bytes(path, MAX_TABLE_BYTES, path.name)
    if len(raw) != size:
        fail(f"{path.name}: protocol size changed while reading")
    if not raw.endswith(b"\n") or b"\r" in raw:
        fail(f"{path.name}: protocol must use final-LF-only framing")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        fail(f"{path.name}: protocol is not UTF-8: {error}")
    lines = text[:-1].split("\n")
    if any(not line for line in lines):
        fail(f"{path.name}: protocol contains a blank line")
    return lines

def rows(path: pathlib.Path, schema: str, columns: list[str]) -> list[dict[str, str]]:
    lines = protocol_lines(path)
    if len(lines) < 3 or lines[0] != schema:
        fail(f"{path.name}: unsupported schema or empty table")
    if lines[1].split("\t") != columns:
        fail(f"{path.name}: unexpected columns")
    parsed: list[dict[str, str]] = []
    for line_number, line in enumerate(lines[2:], start=3):
        fields = line.split("\t")
        if len(fields) != len(columns):
            fail(f"{path.name}:{line_number}: malformed field count")
        parsed.append(dict(zip(columns, fields, strict=True)))
    return parsed

def case_name(value: str, table: str) -> str:
    if CASE_PATTERN.fullmatch(value) is None:
        fail(f"{table}: noncanonical case identifier {value!r}")
    return value

def unique(value: str, seen: set[str], table: str) -> None:
    if value in seen:
        fail(f"{table}: duplicate identifier {value!r}")
    seen.add(value)

def canonical_decimal(value: str) -> bool:
    return value == "0" or (
        bool(value)
        and value[0] in "123456789"
        and value.isascii()
        and value.isdecimal()
    )

def decimal(value: str, field: str, maximum: int) -> int:
    if not canonical_decimal(value):
        fail(f"noncanonical {field}: {value!r}")
    parsed = int(value)
    if parsed > maximum:
        fail(f"{field} exceeds {maximum}: {parsed}")
    return parsed

def decoded_hex(value: str, field: str, maximum_bytes: int, allow_empty: bool = False) -> bytes:
    if not value and not allow_empty:
        fail(f"{field}: empty hexadecimal value")
    if len(value) > maximum_bytes * 2 or len(value) % 2 != 0:
        fail(f"{field}: hexadecimal length is invalid or unbounded")
    if any(character not in LOWER_HEX for character in value):
        fail(f"{field}: hexadecimal value is not canonical lowercase")
    return bytes.fromhex(value)

def safe_source_path(parameter: str) -> pathlib.Path:
    relative = pathlib.PurePosixPath(parameter)
    if relative.is_absolute() or any(part in ("", ".", "..") for part in relative.parts):
        fail(f"unsafe source path: {parameter}")
    path = ROOT.joinpath(*relative.parts).resolve()
    if ROOT not in path.parents or not path.is_file():
        fail(f"source is outside the corpus or is not a file: {parameter}")
    return path

def source_bytes(row: dict[str, str], repetitions: int, declared_length: int) -> bytes:
    kind = row["source_kind"]
    parameter = row["source_parameter"]
    if kind == "empty-v1" and parameter == "-" and repetitions == 1:
        content = b""
    elif kind == "file-v1" and repetitions == 1:
        path = safe_source_path(parameter)
        if path.stat().st_size != declared_length:
            fail(f"{row['case']}: source size differs from its declaration")
        content = bounded_file_bytes(path, MAX_SOURCE_BYTES, row["case"])
    elif kind == "byte-ramp-v1" and parameter == "-" and repetitions > 0:
        generated_length = repetitions * 256
        if generated_length > MAX_SOURCE_BYTES or generated_length != declared_length:
            fail(f"{row['case']}: byte-ramp length is invalid or unbounded")
        content = bytes(range(256)) * repetitions
    else:
        fail(f"{row['case']}: invalid source declaration")
    if len(content) != declared_length:
        fail(f"{row['case']}: source length mismatch")
    return content

def digest(payload: bytes) -> bytes:
    preimage = DATA_MAGIC + VERSION.to_bytes(2, "big") + bytes([ALGORITHM])
    preimage += payload + len(payload).to_bytes(8, "big")
    completed = subprocess.run(
        ["b3sum", "--no-names"],
        input=preimage,
        capture_output=True,
        check=False,
        timeout=30,
    )
    if completed.returncode != 0:
        fail(f"b3sum failed: {completed.stderr.decode(errors='replace').strip()}")
    try:
        output = completed.stdout.decode("ascii")
    except UnicodeDecodeError as error:
        fail(f"b3sum returned non-ASCII output: {error}")
    if not output.endswith("\n") or output.count("\n") != 1:
        fail("b3sum returned noncanonical framing")
    return decoded_hex(output[:-1], "b3sum digest", 32)

def expected_text(length: int, identity_digest: bytes) -> str:
    return f"keep:blob:v1:blake3-256:{length}:{identity_digest.hex()}"

def expected_binary(length: int, identity_digest: bytes) -> bytes:
    return (
        ID_MAGIC + VERSION.to_bytes(2, "big") + bytes([ALGORITHM])
        + length.to_bytes(8, "big") + identity_digest
    )

def check_identities() -> dict[str, IdentityFixture]:
    identity_rows = rows(
        ROOT / "identities.tsv",
        "# keep.golden-file-worldline.identities/v1",
        IDENTITY_COLUMNS,
    )
    fixtures: dict[str, IdentityFixture] = {}
    total = 0
    for row in identity_rows:
        name = case_name(row["case"], "identities.tsv")
        unique(name, set(fixtures), "identities.tsv")
        repetitions = decimal(row["repetitions"], f"{name} repetitions", MAX_SOURCE_BYTES)
        length = decimal(row["logical_length"], f"{name} logical_length", MAX_SOURCE_BYTES)
        total += length
        if total > MAX_TOTAL_BYTES:
            fail("total materialized corpus exceeds bound")
        content = source_bytes(row, repetitions, length)
        identity_digest = digest(content)
        if row["canonical_text"] != expected_text(length, identity_digest):
            fail(f"{name}: canonical text mismatch")
        binary = decoded_hex(row["canonical_binary_hex"], f"{name} binary", BINARY_LENGTH)
        if binary != expected_binary(length, identity_digest):
            fail(f"{name}: canonical binary mismatch")
        fixtures[name] = IdentityFixture(content, row["canonical_text"], binary)
    required = {"empty", "small-text", "binary-ramp", "large-ramp", "state-a", "state-b"}
    if not required.issubset(fixtures):
        fail("identities.tsv: required v1 cases are absent")
    expected_b = fixtures["state-a"].content[:6] + b"INSERTED\n" + fixtures["state-a"].content[6:]
    if fixtures["state-b"].content != expected_b:
        fail("state-b is not the declared insertion into state-a")
    return fixtures

def text_outcome(encoded: bytes) -> str | None:
    try:
        encoded.decode("utf-8")
    except UnicodeDecodeError as error:
        fail(f"invalid-text.tsv: input is not UTF-8: {error}")
    if len(encoded) > MAX_TEXT_BYTES:
        return "keep.identity.input_too_long"
    fields = encoded.split(b":")
    if len(fields) < 6 or any(not field for field in fields[:6]):
        return "keep.identity.malformed_structure"
    if len(fields) > 6:
        return "keep.identity.trailing_data"
    scheme, kind, version, algorithm, length, identity_digest = fields
    if scheme != b"keep":
        return "keep.identity.invalid_scheme"
    if kind != b"blob":
        return "keep.identity.invalid_kind"
    if version != b"v1":
        number = version[1:] if version.startswith(b"v") else b""
        if not canonical_decimal_bytes(number) or int(number) > U16_MAX:
            return "keep.identity.malformed_version"
        return "keep.identity.unsupported_version"
    if algorithm != b"blake3-256":
        return "keep.identity.unsupported_algorithm"
    if not canonical_decimal_bytes(length):
        return "keep.identity.noncanonical_length"
    if int(length) > U64_MAX:
        return "keep.identity.length_overflow"
    if len(identity_digest) != 64:
        return "keep.identity.invalid_digest_length"
    for character in identity_digest:
        if ord("A") <= character <= ord("F"):
            return "keep.identity.noncanonical_digest_case"
        if character not in b"0123456789abcdef":
            return "keep.identity.invalid_digest_alphabet"
    return None

def canonical_decimal_bytes(value: bytes) -> bool:
    return value == b"0" or (
        bool(value)
        and ord("1") <= value[0] <= ord("9")
        and all(ord("0") <= digit <= ord("9") for digit in value)
    )

def check_invalid_text() -> None:
    invalid_rows = rows(
        ROOT / "invalid-text.tsv",
        "# keep.golden-file-worldline.invalid-text/v1",
        INVALID_TEXT_COLUMNS,
    )
    seen: set[str] = set()
    outcomes: set[str] = set()
    for row in invalid_rows:
        name = case_name(row["case"], "invalid-text.tsv")
        unique(name, seen, "invalid-text.tsv")
        encoded = decoded_hex(row["input_hex"], f"{name} input", MAX_INVALID_TEXT_BYTES, True)
        observed = text_outcome(encoded)
        if observed is None or row["expected_outcome"] != observed:
            fail(f"{name}: expected text outcome does not match {observed!r}")
        outcomes.add(observed)
    if not REQUIRED_TEXT_OUTCOMES.issubset(outcomes):
        fail("invalid-text.tsv: required v1 outcome coverage is absent")

def mutate(target: bytes, operation: str, offset: int, value_field: str, name: str) -> bytes:
    changed = bytearray(target)
    if operation == "truncate":
        if value_field != "-" or offset >= len(changed):
            fail(f"{name}: invalid truncation")
        del changed[offset:]
    elif operation == "append":
        value = decoded_hex(value_field, f"{name} value", MAX_MUTATION_VALUE_BYTES)
        if offset != len(changed):
            fail(f"{name}: append offset does not equal target length")
        changed.extend(value)
    else:
        width = {"xor-byte": 1, "set-u8": 1, "set-u16-be": 2}.get(operation)
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
    if result == target or len(result) > MAX_SOURCE_BYTES + MAX_MUTATION_VALUE_BYTES:
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

def check_mutations(fixtures: dict[str, IdentityFixture]) -> None:
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
        target = fixture.content if target_kind == "content" else fixture.canonical_binary
        offset = decimal(row["offset"], f"{name} offset", len(target))
        changed = mutate(target, row["operation"], offset, row["value_hex"], name)
        if target_kind == "content":
            observed = expected_text(len(changed), digest(changed))
            if row["expected_outcome"] != "keep.content.mismatch" or observed == fixture.canonical_text:
                fail(f"{name}: content mutation does not prove mismatch")
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
        number = decimal(row["step"], "scenario step", len(step_rows))
        if number != expected_number:
            fail("steps.tsv: step numbers are not canonical and contiguous")
        operation = row["operation"]
        outcome = STEP_OUTCOMES.get(operation)
        if outcome is None or row["expected_outcome"] != outcome:
            fail(f"steps.tsv:{number}: operation outcome is invalid")
        identity = fixtures.get(row["identity_case"])
        if identity is None:
            fail(f"steps.tsv:{number}: identity case is absent")
        if operation == "read-absent":
            if row["input_case"] != "-" or identity.canonical_binary in admitted:
                fail(f"steps.tsv:{number}: absent read is not absent")
        else:
            source = fixtures.get(row["input_case"])
            if source is None:
                fail(f"steps.tsv:{number}: input case is absent")
            same_identity = source.canonical_binary == identity.canonical_binary
            if operation == "verify-claimed-content" and same_identity:
                fail(f"steps.tsv:{number}: mismatch uses matching content")
            if operation != "verify-claimed-content" and not same_identity:
                fail(f"steps.tsv:{number}: exact operation substitutes identity")
            if operation == "admit-exact":
                admitted.add(identity.canonical_binary)
            if operation == "read-exact" and identity.canonical_binary not in admitted:
                fail(f"steps.tsv:{number}: exact read precedes admission")
        operations.append(operation)
    if operations != REQUIRED_STEP_OPERATIONS:
        fail("steps.tsv: ordered Golden File Worldline v1 operations moved")

def issue_numbers(value: str, capability: str) -> tuple[int, ...]:
    parts = value.split(",")
    issues = tuple(decimal(part, f"{capability} issue", U64_MAX) for part in parts)
    if not issues or any(issue == 0 for issue in issues) or tuple(sorted(set(issues))) != issues:
        fail(f"{capability}: owning issues are empty, duplicate, or unordered")
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
            fail(f"capabilities.tsv: invalid coordinate {capability!r}")
        unique(capability, seen, "capabilities.tsv")
        milestone = row["first_milestone"]
        if not milestone.startswith("M"):
            fail(f"{capability}: malformed milestone")
        observed = (
            row["posture"],
            decimal(milestone[1:], f"{capability} milestone", U16_MAX),
            issue_numbers(row["owning_issues"], capability),
        )
        expected = CAPABILITY_CONTRACTS.get(capability)
        if expected is None or observed != expected:
            fail(f"{capability}: capability contract moved")
        if not row["claim"] or row["claim"] != row["claim"].strip():
            fail(f"{capability}: claim is empty or noncanonical")
    if seen != set(CAPABILITY_CONTRACTS):
        fail("capabilities.tsv: required v1 capability set moved")

def main() -> None:
    fixtures = check_identities()
    check_invalid_text()
    check_mutations(fixtures)
    check_steps(fixtures)
    check_capabilities()
    print("Golden File Worldline v1 corpus is exact.")

if __name__ == "__main__":
    try:
        main()
    except subprocess.TimeoutExpired as error:
        fail(f"b3sum exceeded {error.timeout} seconds")
    except FileNotFoundError as error:
        fail(f"required file or b3sum executable not found: {error.filename}")
    except OSError as error:
        fail(f"operating-system failure: {error}")

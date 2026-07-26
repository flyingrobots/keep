"""Independent Golden File Worldline identity construction and parsing."""

from __future__ import annotations

from dataclasses import dataclass
import subprocess

from corpus_protocol import (
    MAX_SOURCE_BYTES,
    ROOT,
    U16_MAX,
    U64_MAX,
    bounded_file_bytes,
    case_name,
    decimal,
    decoded_hex,
    fail,
    rows,
    safe_source_path,
    unique,
)

DATA_MAGIC = b"KEEP:BLOB:DATA\0\0"
ID_MAGIC = b"KEEP:BLOB:ID\0\0\0\0"
VERSION = 1
ALGORITHM = 1
BINARY_LENGTH = 59
MAX_TEXT_BYTES = 109
MAX_TOTAL_BYTES = 1_048_911
MAX_INVALID_TEXT_BYTES = 4_096

IDENTITY_COLUMNS = [
    "case",
    "source_kind",
    "source_parameter",
    "repetitions",
    "logical_length",
    "canonical_text",
    "canonical_binary_hex",
]
INVALID_TEXT_COLUMNS = ["case", "input_hex", "expected_outcome"]
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


@dataclass(frozen=True)
class IdentityFixture:
    content: bytes
    canonical_text: str
    canonical_binary: bytes


def source_bytes(
    row: dict[str, str],
    repetitions: int,
    declared_length: int,
) -> bytes:
    kind = row["source_kind"]
    parameter = row["source_parameter"]
    if kind == "empty-v1" and parameter == "-" and repetitions == 1:
        content = b""
    elif kind == "file-v1" and repetitions == 1:
        path = safe_source_path(parameter)
        if path.stat().st_size != declared_length:
            fail(
                f"{row['case']}: source size differs from its declaration"
            )
        content = bounded_file_bytes(
            path,
            MAX_SOURCE_BYTES,
            row["case"],
        )
    elif kind == "byte-ramp-v1" and parameter == "-" and repetitions > 0:
        generated_length = repetitions * 256
        if (
            generated_length > MAX_SOURCE_BYTES
            or generated_length != declared_length
        ):
            fail(
                f"{row['case']}: byte-ramp length is invalid or unbounded"
            )
        content = bytes(range(256)) * repetitions
    else:
        fail(f"{row['case']}: invalid source declaration")
    if len(content) != declared_length:
        fail(f"{row['case']}: source length mismatch")
    return content


def digest(payload: bytes) -> bytes:
    preimage = (
        DATA_MAGIC
        + VERSION.to_bytes(2, "big")
        + bytes([ALGORITHM])
    )
    preimage += payload + len(payload).to_bytes(8, "big")
    completed = subprocess.run(
        ["b3sum", "--no-names"],
        input=preimage,
        capture_output=True,
        check=False,
        timeout=30,
    )
    if completed.returncode != 0:
        fail(
            "b3sum failed: "
            f"{completed.stderr.decode(errors='replace').strip()}"
        )
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
        ID_MAGIC
        + VERSION.to_bytes(2, "big")
        + bytes([ALGORITHM])
        + length.to_bytes(8, "big")
        + identity_digest
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
        repetitions = decimal(
            row["repetitions"],
            f"{name} repetitions",
            MAX_SOURCE_BYTES,
        )
        length = decimal(
            row["logical_length"],
            f"{name} logical_length",
            MAX_SOURCE_BYTES,
        )
        total += length
        if total > MAX_TOTAL_BYTES:
            fail("total materialized corpus exceeds bound")
        content = source_bytes(row, repetitions, length)
        identity_digest = digest(content)
        if row["canonical_text"] != expected_text(
            length,
            identity_digest,
        ):
            fail(f"{name}: canonical text mismatch")
        binary = decoded_hex(
            row["canonical_binary_hex"],
            f"{name} binary",
            BINARY_LENGTH,
        )
        if binary != expected_binary(length, identity_digest):
            fail(f"{name}: canonical binary mismatch")
        fixtures[name] = IdentityFixture(
            content,
            row["canonical_text"],
            binary,
        )
    required = {
        "empty",
        "small-text",
        "binary-ramp",
        "large-ramp",
        "state-a",
        "state-b",
    }
    if not required.issubset(fixtures):
        fail("identities.tsv: required v1 cases are absent")
    expected_b = (
        fixtures["state-a"].content[:6]
        + b"INSERTED\n"
        + fixtures["state-a"].content[6:]
    )
    if fixtures["state-b"].content != expected_b:
        fail("state-b is not the declared insertion into state-a")
    return fixtures


def canonical_decimal_bytes(value: bytes) -> bool:
    return value == b"0" or (
        bool(value)
        and ord("1") <= value[0] <= ord("9")
        and all(ord("0") <= digit <= ord("9") for digit in value)
    )


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
        encoded = decoded_hex(
            row["input_hex"],
            f"{name} input",
            MAX_INVALID_TEXT_BYTES,
            True,
        )
        observed = text_outcome(encoded)
        if observed is None or row["expected_outcome"] != observed:
            fail(
                f"{name}: expected text outcome does not match "
                f"{observed!r}"
            )
        outcomes.add(observed)
    if not REQUIRED_TEXT_OUTCOMES.issubset(outcomes):
        fail(
            "invalid-text.tsv: required v1 outcome coverage is absent"
        )

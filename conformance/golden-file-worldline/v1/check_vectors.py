#!/usr/bin/env python3
"""Check Golden File Worldline vectors without importing Keep."""

from __future__ import annotations

import csv
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parent
DATA_MAGIC = b"KEEP:BLOB:DATA\0\0"
ID_MAGIC = b"KEEP:BLOB:ID\0\0\0\0"
VERSION = 1
ALGORITHM = 1
BINARY_LENGTH = 59
MAX_SOURCE_BYTES = 1_048_576
MAX_TOTAL_BYTES = 1_048_911


def fail(message: str) -> None:
    raise SystemExit(f"golden vector check failed: {message}")


def rows(path: pathlib.Path, schema: str, columns: list[str]) -> list[dict[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if len(lines) < 2 or lines[0] != schema:
        fail(f"{path.name}: unsupported schema")
    reader = csv.DictReader(lines[1:], delimiter="\t")
    if reader.fieldnames != columns:
        fail(f"{path.name}: unexpected columns")
    parsed = list(reader)
    if any(None in row or None in row.values() for row in parsed):
        fail(f"{path.name}: malformed row")
    return parsed


def source_bytes(row: dict[str, str]) -> bytes:
    kind = row["source_kind"]
    parameter = row["source_parameter"]
    repetitions = parse_decimal(row["repetitions"], "repetitions")
    if kind == "empty-v1" and parameter == "-" and repetitions == 1:
        return b""
    if kind == "file-v1" and repetitions == 1:
        path = safe_source_path(parameter)
        return path.read_bytes()
    if kind == "byte-ramp-v1" and parameter == "-":
        return bytes(range(256)) * repetitions
    fail(f"{row['case']}: invalid source declaration")


def safe_source_path(parameter: str) -> pathlib.Path:
    relative = pathlib.PurePosixPath(parameter)
    if relative.is_absolute() or any(part in ("", ".", "..") for part in relative.parts):
        fail(f"unsafe source path: {parameter}")
    path = ROOT.joinpath(*relative.parts).resolve()
    if ROOT not in path.parents:
        fail(f"source escapes corpus: {parameter}")
    return path


def parse_decimal(value: str, field: str) -> int:
    if value == "0":
        return 0
    if not value or value[0] == "0" or not value.isascii() or not value.isdecimal():
        fail(f"noncanonical {field}: {value!r}")
    return int(value)


def digest(payload: bytes) -> bytes:
    preimage = DATA_MAGIC + VERSION.to_bytes(2, "big") + bytes([ALGORITHM])
    preimage += payload + len(payload).to_bytes(8, "big")
    completed = subprocess.run(
        ["b3sum", "--no-names"],
        input=preimage,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        fail(f"b3sum failed: {completed.stderr.decode(errors='replace').strip()}")
    try:
        return bytes.fromhex(completed.stdout.decode("ascii").strip())
    except (UnicodeDecodeError, ValueError) as error:
        fail(f"b3sum returned invalid hex: {error}")


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


def check_identities() -> None:
    identity_rows = rows(
        ROOT / "identities.tsv",
        "# keep.golden-file-worldline.identities/v1",
        [
            "case",
            "source_kind",
            "source_parameter",
            "repetitions",
            "logical_length",
            "canonical_text",
            "canonical_binary_hex",
        ],
    )
    seen: set[str] = set()
    total = 0
    loaded: dict[str, bytes] = {}
    for row in identity_rows:
        case = row["case"]
        if not case or case in seen:
            fail(f"duplicate or empty case: {case!r}")
        seen.add(case)
        payload = source_bytes(row)
        loaded[case] = payload
        total += len(payload)
        declared_length = parse_decimal(row["logical_length"], "logical_length")
        if len(payload) != declared_length or len(payload) > MAX_SOURCE_BYTES:
            fail(f"{case}: source length mismatch or limit exceeded")
        observed_digest = digest(payload)
        if row["canonical_text"] != expected_text(len(payload), observed_digest):
            fail(f"{case}: canonical text mismatch")
        try:
            binary = bytes.fromhex(row["canonical_binary_hex"])
        except ValueError as error:
            fail(f"{case}: invalid binary hex: {error}")
        if len(binary) != BINARY_LENGTH or binary != expected_binary(len(payload), observed_digest):
            fail(f"{case}: canonical binary mismatch")
    if total > MAX_TOTAL_BYTES:
        fail("total materialized corpus exceeds bound")
    expected_b = loaded["state-a"][:6] + b"INSERTED\n" + loaded["state-a"][6:]
    if loaded["state-b"] != expected_b:
        fail("state-b is not the declared insertion into state-a")


def check_table_shapes() -> None:
    rows(
        ROOT / "steps.tsv",
        "# keep.golden-file-worldline.steps/v1",
        ["step", "operation", "input_case", "identity_case", "expected_outcome"],
    )
    rows(
        ROOT / "invalid-text.tsv",
        "# keep.golden-file-worldline.invalid-text/v1",
        ["case", "input_hex", "expected_outcome"],
    )
    rows(
        ROOT / "mutations.tsv",
        "# keep.golden-file-worldline.mutations/v1",
        [
            "case",
            "target_kind",
            "target_case",
            "operation",
            "offset",
            "value_hex",
            "expected_outcome",
        ],
    )
    rows(
        ROOT / "capabilities.tsv",
        "# keep.golden-file-worldline.capabilities/v1",
        ["capability", "posture", "first_milestone", "owning_issues", "claim"],
    )


def main() -> None:
    check_identities()
    check_table_shapes()
    print("Golden File Worldline v1 vectors are exact.")


if __name__ == "__main__":
    try:
        main()
    except FileNotFoundError as error:
        fail(f"required file or b3sum executable not found: {error.filename}")
    except OSError as error:
        fail(f"operating-system failure: {error}")

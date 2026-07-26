#!/usr/bin/env python3
"""Validate Keep's bounded ChunkId version-1 corpus independently."""

from __future__ import annotations

import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import NoReturn

ROOT = Path(__file__).resolve().parent
IDENTITIES = ROOT / "identities.tsv"
SCHEMA = "keep.chunk-identities/v1"
COLUMNS = "case\trecipe\tparameter\tcount\tchunk_length\tdigest_hex"
DATA_MAGIC = b"KEEP:CHUNK:DATA\0"
VERSION = (1).to_bytes(2, "big")
ALGORITHM = bytes([1])
MAX_CASES = 16
MAX_CHUNK_BYTES = 262_144
MAX_TOTAL_BYTES = 300_000
MAX_TABLE_BYTES = 1_048_576
B3SUM_TIMEOUT_SECONDS = 10
LOWER_HEX = re.compile(r"[0-9a-f]+").fullmatch
CASE_NAME = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*").fullmatch
REQUIRED_CASES = frozenset({"one-zero", "sample-text", "maximum-zeros"})
EXPECTED_TOTAL_BYTES = 262_163


@dataclass(frozen=True)
class IdentityCase:
    name: str
    payload: bytes
    expected_digest: str


def fail(message: str) -> NoReturn:
    raise SystemExit(f"chunk identity corpus check failed: {message}")


def resolve_b3sum() -> str:
    candidate = shutil.which("b3sum")
    if candidate is None:
        fail("b3sum was not found on PATH")
    try:
        resolved = Path(candidate).resolve(strict=True)
    except OSError as error:
        fail(f"cannot resolve b3sum: {error}")
    if not resolved.is_file():
        fail("resolved b3sum path is not a file")
    return str(resolved)


B3SUM = resolve_b3sum()


def bounded_bytes(path: Path) -> bytes:
    try:
        size = path.stat().st_size
        if size == 0 or size > MAX_TABLE_BYTES:
            fail(f"{path.name} size is outside its bound")
        with path.open("rb") as source:
            content = source.read(MAX_TABLE_BYTES + 1)
    except OSError as error:
        fail(f"cannot read {path.name}: {error}")
    if len(content) != size or len(content) > MAX_TABLE_BYTES:
        fail(f"{path.name} changed size or exceeded its bound while reading")
    return content


def canonical_decimal(value: str, field: str) -> int:
    if value != "0" and (not value or value.startswith("0")):
        fail(f"{field} is noncanonical")
    if not value.isascii() or not value.isdigit():
        fail(f"{field} is not unsigned decimal")
    return int(value)


def lowercase_hex(value: str, field: str, exact_bytes: int | None = None) -> bytes:
    if not value or LOWER_HEX(value) is None or len(value) % 2 != 0:
        fail(f"{field} is not canonical lowercase hexadecimal")
    decoded = bytes.fromhex(value)
    if exact_bytes is not None and len(decoded) != exact_bytes:
        fail(f"{field} has the wrong width")
    return decoded


def payload_for(recipe: str, parameter: str, count: int) -> bytes:
    if recipe == "repeated-byte-v1":
        pattern = lowercase_hex(parameter, "repeated-byte parameter", 1)
    elif recipe == "hex-repeat-v1":
        pattern = lowercase_hex(parameter, "hex-repeat parameter")
    else:
        fail(f"unsupported recipe {recipe!r}")
    declared = len(pattern) * count
    if declared > MAX_CHUNK_BYTES:
        fail("recipe exceeds the fixture chunk bound")
    return pattern * count


def read_cases() -> list[IdentityCase]:
    raw = bounded_bytes(IDENTITIES)
    if not raw.endswith(b"\n") or raw.endswith(b"\n\n") or b"\r" in raw or b"\0" in raw:
        fail("identities.tsv has noncanonical framing")
    lines = raw.decode("utf-8").splitlines()
    if len(lines) < 2 or lines[0] != SCHEMA or lines[1] != COLUMNS:
        fail("identities.tsv header mismatch")
    rows = lines[2:]
    if not rows or len(rows) > MAX_CASES:
        fail("identity case count is outside its bound")
    cases: list[IdentityCase] = []
    names: set[str] = set()
    total = 0
    for row in rows:
        fields = row.split("\t")
        if len(fields) != 6:
            fail("identity row has the wrong field count")
        name, recipe, parameter, count_text, length_text, digest = fields
        if CASE_NAME(name) is None or name in names:
            fail("identity case name is invalid or duplicated")
        names.add(name)
        count = canonical_decimal(count_text, "recipe count")
        declared_length = canonical_decimal(length_text, "chunk length")
        payload = payload_for(recipe, parameter, count)
        if not payload or len(payload) != declared_length:
            fail("payload length does not match the nonempty declaration")
        lowercase_hex(digest, "expected digest", 32)
        total += len(payload)
        if total > MAX_TOTAL_BYTES:
            fail("identity corpus exceeds its aggregate bound")
        cases.append(IdentityCase(name, payload, digest))
    if names != REQUIRED_CASES or total != EXPECTED_TOTAL_BYTES:
        fail("required identity witnesses or aggregate length moved")
    return cases


def digest(payload: bytes) -> str:
    preimage = DATA_MAGIC + VERSION + ALGORITHM + payload + len(payload).to_bytes(4, "big")
    try:
        completed = subprocess.run(
            [B3SUM, "--no-names"],
            input=preimage,
            capture_output=True,
            check=False,
            timeout=B3SUM_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        fail(f"b3sum exceeded {error.timeout} seconds")
    except OSError as error:
        fail(f"cannot execute b3sum: {error}")
    if completed.returncode != 0:
        fail(f"b3sum failed: {completed.stderr.decode(errors='replace').strip()}")
    output = completed.stdout
    if len(output) != 65 or not output.endswith(b"\n"):
        fail("b3sum returned noncanonical framing")
    try:
        encoded = output[:-1].decode("ascii")
    except UnicodeDecodeError as error:
        fail(f"b3sum returned non-ASCII output: {error}")
    lowercase_hex(encoded, "b3sum digest", 32)
    return encoded


def main() -> None:
    cases = read_cases()
    for case in cases:
        observed = digest(case.payload)
        if observed != case.expected_digest:
            fail(f"ChunkId digest moved for {case.name}")
    total = sum(len(case.payload) for case in cases)
    print(f"ChunkId v1 corpus is exact: {len(cases)} cases, {total} source bytes")


if __name__ == "__main__":
    main()

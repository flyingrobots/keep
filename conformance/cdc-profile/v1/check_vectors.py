#!/usr/bin/env python3
"""Validate the CDC profile v1 corpus without importing Keep."""

from __future__ import annotations

import hashlib
import itertools
import pathlib
import re
import struct
import subprocess

from scalar_fastcdc import (
    LONG_MASK,
    MAXIMUM,
    MINIMUM,
    NORMALIZATION,
    SEED,
    SHORT_MASK,
    STATE_WIDTH,
    TARGET,
    StreamingChunker,
    boundary_adjacent_parts,
    probe_fingerprint,
    reference_boundaries,
    scheduled_parts,
)

ROOT = pathlib.Path(__file__).resolve().parent
PROFILE_NAME = "fastcdc-64k-v1"
GEAR_MAGIC = b"KEEP:GEAR:TABLE\0"
PROFILE_MAGIC = b"KEEP:CDC:PROFILE"
FORMAT_VERSION = 1
HASH_ALGORITHM = 1
BOUNDARY_ALGORITHM = 1
PROFILE_LENGTH = 96
U64_MASK = (1 << 64) - 1
TABLE_BYTES = 2_048
MAX_PROTOCOL_BYTES = 1_048_576
MAX_PROTOCOL_ROWS = 256
MAX_SOURCE_BYTES = 2_097_152
MAX_INPUT_BYTES = MAX_EDIT_BYTES = 4_096
B3SUM_TIMEOUT_SECONDS = 30
CASE_PATTERN = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*\Z")
HEX_PATTERN = re.compile(r"[0-9a-f]*\Z")

PROFILE_COLUMNS = [
    "profile", "gear_table", "gear_checksum_hex", "profile_record",
    "storage_profile_id",
]
SOURCE_COLUMNS = ["case", "recipe", "parameter", "count", "logical_length"]
MUTATION_COLUMNS = [
    "case", "base_case", "operation", "offset", "span_length",
    "value_hex", "logical_length",
]
BOUNDARY_COLUMNS = ["case", "chunk_count", "boundaries"]
REQUIRED_CASES = {
    "empty", "tiny", "min-minus-one", "min-exact", "min-plus-one",
    "target-minus-one", "target-exact", "target-plus-one",
    "max-minus-one", "max-exact", "max-plus-one", "zeros-long",
    "ff-long", "alternating-long", "random-long", "source-like",
    "probe-byte-carry", "short-mask-match", "natural-cut-runt",
    "edit-base", "early-insert", "early-delete", "early-xor",
    "target-long-transition",
}
SOURCE_CASES = REQUIRED_CASES - {
    "early-insert", "early-delete", "early-xor", "target-long-transition",
}
MUTATION_CASES = REQUIRED_CASES - SOURCE_CASES
MAX_CORPUS_SOURCE_BYTES = len(REQUIRED_CASES) * MAX_SOURCE_BYTES


def fail(message: str) -> None:
    raise SystemExit(f"CDC corpus check failed: {message}")


def bounded_bytes(path: pathlib.Path, maximum: int, label: str) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        fail(f"{label}: cannot stat: {error}")
    if size > maximum:
        fail(f"{label}: {size} bytes exceeds {maximum}")
    try:
        with path.open("rb") as source:
            content = source.read(maximum + 1)
    except OSError as error:
        fail(f"{label}: cannot read: {error}")
    if len(content) != size or len(content) > maximum:
        fail(f"{label}: size changed or exceeded its bound while reading")
    return content


def protocol_lines(path: pathlib.Path) -> list[str]:
    raw = bounded_bytes(path, MAX_PROTOCOL_BYTES, path.name)
    if not raw or not raw.endswith(b"\n") or b"\r" in raw:
        fail(f"{path.name}: protocol must use non-empty final-LF-only framing")
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
    if len(lines) - 2 > MAX_PROTOCOL_ROWS:
        fail(f"{path.name}: row count exceeds {MAX_PROTOCOL_ROWS}")
    parsed: list[dict[str, str]] = []
    for line_number, line in enumerate(lines[2:], start=3):
        fields = line.split("\t")
        if len(fields) != len(columns) or any(field == "" for field in fields):
            fail(f"{path.name}:{line_number}: malformed field count or empty field")
        parsed.append(dict(zip(columns, fields, strict=True)))
    return parsed


def canonical_decimal(value: str, field: str, maximum: int) -> int:
    if len(value) > len(str(maximum)) or value != "0" and (
        not value or value[0] == "0" or not value.isascii() or not value.isdecimal()
    ):
        fail(f"{field}: noncanonical decimal {value!r}")
    result = int(value)
    if result > maximum:
        fail(f"{field}: {result} exceeds {maximum}")
    return result


def lowercase_hex(value: str, field: str, exact_bytes: int | None = None) -> bytes:
    if len(value) % 2 or HEX_PATTERN.fullmatch(value) is None:
        fail(f"{field}: hexadecimal value is not canonical lowercase")
    decoded = bytes.fromhex(value)
    if exact_bytes is not None and len(decoded) != exact_bytes:
        fail(f"{field}: expected {exact_bytes} bytes, observed {len(decoded)}")
    return decoded


def case_name(value: str, table: str) -> str:
    if CASE_PATTERN.fullmatch(value) is None:
        fail(f"{table}: noncanonical case name {value!r}")
    return value


def safe_path(value: str, maximum: int) -> pathlib.Path:
    relative = pathlib.PurePosixPath(value)
    if relative.is_absolute() or any(part in ("", ".", "..") for part in relative.parts):
        fail(f"unsafe corpus path: {value!r}")
    path = ROOT.joinpath(*relative.parts).resolve()
    if ROOT not in path.parents or not path.is_file():
        fail(f"corpus path is outside the corpus or not a file: {value!r}")
    if path.stat().st_size > maximum:
        fail(f"corpus path exceeds {maximum} bytes: {value!r}")
    return path


def blake3(payload: bytes) -> bytes:
    try:
        completed = subprocess.run(
            ["b3sum", "--no-names"], input=payload, capture_output=True,
            check=False, timeout=B3SUM_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired:
        fail(f"b3sum exceeded {B3SUM_TIMEOUT_SECONDS} seconds")
    except OSError as error:
        fail(f"cannot execute b3sum: {error}")
    if completed.returncode != 0:
        fail(f"b3sum failed: {completed.stderr.decode(errors='replace').strip()}")
    output = completed.stdout
    if len(output) != 65 or output[-1:] != b"\n":
        fail("b3sum returned noncanonical framing")
    try:
        return lowercase_hex(output[:-1].decode("ascii"), "b3sum digest", 32)
    except UnicodeDecodeError as error:
        fail(f"b3sum returned non-ASCII output: {error}")


def generated_gear_table() -> bytes:
    entries = []
    for value in range(256):
        # MD5 is a deterministic table recipe here, never a security primitive.
        entries.append(
            hashlib.md5(bytes([value]) * 64, usedforsecurity=False).digest()[:8]
        )
    return b"".join(entries)


def typed_gear_checksum(table: bytes) -> bytes:
    preimage = GEAR_MAGIC + struct.pack(">HB", FORMAT_VERSION, HASH_ALGORITHM)
    preimage += table + struct.pack(">Q", len(table))
    return blake3(preimage)


def canonical_profile_record(gear_checksum: bytes) -> bytes:
    record = PROFILE_MAGIC
    record += struct.pack(">HHHH", FORMAT_VERSION, PROFILE_LENGTH, BOUNDARY_ALGORITHM, 0)
    record += gear_checksum
    record += struct.pack(">QIII", SEED, MINIMUM, TARGET, MAXIMUM)
    record += struct.pack(">BBH", NORMALIZATION, STATE_WIDTH, 0)
    record += struct.pack(">QQ", SHORT_MASK, LONG_MASK)
    if len(record) != PROFILE_LENGTH:
        fail(f"constructed profile has {len(record)} bytes, expected {PROFILE_LENGTH}")
    return record


def validate_profile() -> tuple[int, ...]:
    profile_rows = rows(ROOT / "profile.tsv", "keep.cdc-profile-fixture/v1", PROFILE_COLUMNS)
    if len(profile_rows) != 1 or profile_rows[0]["profile"] != PROFILE_NAME:
        fail("profile.tsv must contain exactly the canonical profile")
    row = profile_rows[0]
    table = bounded_bytes(safe_path(row["gear_table"], TABLE_BYTES), TABLE_BYTES, "gear table")
    generated = generated_gear_table()
    if len(table) != TABLE_BYTES or table != generated:
        fail("authoritative Gear table differs from its reproducible recipe")
    checksum = typed_gear_checksum(table)
    if checksum != lowercase_hex(row["gear_checksum_hex"], "Gear checksum", 32):
        fail("typed Gear checksum differs from profile.tsv")
    record = bounded_bytes(safe_path(row["profile_record"], PROFILE_LENGTH), PROFILE_LENGTH, "profile")
    expected_record = canonical_profile_record(checksum)
    if record != expected_record:
        fail("profile record differs from its canonical field encoding")
    digest = blake3(record)
    expected_id = f"keep:storage-profile:v1:blake3-256:{digest.hex()}"
    if row["storage_profile_id"] != expected_id:
        fail("StorageProfileId differs from the profile record digest")
    return tuple(struct.unpack(">256Q", table))


def xorshift64(seed: int, count: int) -> bytes:
    if seed == 0:
        fail("xorshift64-v1 seed must be nonzero")
    output = bytearray(count)
    state = seed
    for index in range(count):
        state ^= (state << 13) & U64_MASK
        state ^= state >> 7
        state ^= (state << 17) & U64_MASK
        state &= U64_MASK
        output[index] = state & 0xFF
    return bytes(output)


def primitive_source(row: dict[str, str]) -> bytes:
    recipe = row["recipe"]
    parameter = row["parameter"]
    count = canonical_decimal(row["count"], f"{row['case']} count", MAX_SOURCE_BYTES)
    declared = canonical_decimal(row["logical_length"], f"{row['case']} length", MAX_SOURCE_BYTES)
    if recipe == "empty-v1" and parameter == "-" and count == 0:
        content = b""
    elif recipe == "repeated-byte-v1":
        content = lowercase_hex(parameter, f"{row['case']} byte", 1) * count
    elif recipe == "alternating-v1":
        pattern = lowercase_hex(parameter, f"{row['case']} pattern", 2)
        if count > MAX_SOURCE_BYTES // len(pattern):
            fail(f"{row['case']}: repeated pattern exceeds source bound")
        content = pattern * count
    elif recipe == "xorshift64-v1":
        seed = int.from_bytes(lowercase_hex(parameter, f"{row['case']} seed", 8), "big")
        content = xorshift64(seed, count)
    elif recipe == "file-repeat-v1":
        unit = bounded_bytes(safe_path(parameter, MAX_INPUT_BYTES), MAX_INPUT_BYTES, row["case"])
        if not unit or count > MAX_SOURCE_BYTES // len(unit):
            fail(f"{row['case']}: repeated file is empty or exceeds source bound")
        content = unit * count
    else:
        fail(f"{row['case']}: unsupported or malformed recipe {recipe!r}")
    if len(content) != declared:
        fail(f"{row['case']}: declared {declared} bytes, generated {len(content)}")
    return content


def load_sources() -> dict[str, bytes]:
    loaded: dict[str, bytes] = {}
    aggregate_bytes = 0
    for row in rows(ROOT / "sources.tsv", "keep.cdc-sources/v1", SOURCE_COLUMNS):
        name = case_name(row["case"], "sources.tsv")
        if name in loaded or name not in SOURCE_CASES:
            fail(f"sources.tsv: duplicate or unknown case {name!r}")
        content = primitive_source(row)
        aggregate_bytes += len(content)
        if aggregate_bytes > MAX_CORPUS_SOURCE_BYTES:
            fail("source corpus exceeds its aggregate byte bound")
        loaded[name] = content
    for row in rows(ROOT / "mutations.tsv", "keep.cdc-mutations/v1", MUTATION_COLUMNS):
        name = case_name(row["case"], "mutations.tsv")
        base_name = case_name(row["base_case"], "mutations.tsv")
        if name in loaded or name not in MUTATION_CASES or base_name not in loaded:
            fail(f"mutations.tsv: duplicate/unknown case or absent base {name!r}")
        base = loaded[base_name]
        offset = canonical_decimal(row["offset"], f"{name} offset", len(base))
        span = canonical_decimal(row["span_length"], f"{name} span", MAX_EDIT_BYTES)
        declared = canonical_decimal(row["logical_length"], f"{name} length", MAX_SOURCE_BYTES)
        value = b"" if row["value_hex"] == "-" else lowercase_hex(row["value_hex"], f"{name} value")
        if len(value) > MAX_EDIT_BYTES or offset + span > len(base):
            fail(f"{name}: edit is outside its bounded base")
        operation = row["operation"]
        if operation == "insert-v1" and span == 0 and value:
            content = base[:offset] + value + base[offset:]
        elif operation == "delete-v1" and span > 0 and not value:
            content = base[:offset] + base[offset + span:]
        elif operation == "xor-v1" and span > 0 and len(value) == span:
            changed = bytes(left ^ right for left, right in zip(base[offset:offset + span], value, strict=True))
            content = base[:offset] + changed + base[offset + span:]
        else:
            fail(f"{name}: malformed mutation {operation!r}")
        if len(content) != declared:
            fail(f"{name}: declared {declared} bytes, generated {len(content)}")
        aggregate_bytes += len(content)
        if aggregate_bytes > MAX_CORPUS_SOURCE_BYTES:
            fail("source corpus exceeds its aggregate byte bound")
        loaded[name] = content
    return loaded


def parse_boundaries(sources: dict[str, bytes]) -> dict[str, tuple[int, ...]]:
    parsed: dict[str, tuple[int, ...]] = {}
    for row in rows(ROOT / "boundaries.tsv", "keep.cdc-boundaries/v1", BOUNDARY_COLUMNS):
        name = case_name(row["case"], "boundaries.tsv")
        if name in parsed or name not in sources:
            fail(f"boundaries.tsv: duplicate or unknown case {name!r}")
        count = canonical_decimal(row["chunk_count"], f"{name} chunk count", MAX_SOURCE_BYTES)
        if row["boundaries"] == "-":
            ends: tuple[int, ...] = ()
        else:
            ends = tuple(
                canonical_decimal(value, f"{name} boundary", len(sources[name]))
                for value in row["boundaries"].split(",")
            )
        if len(ends) != count:
            fail(f"{name}: declared {count} chunks, recorded {len(ends)}")
        parsed[name] = ends
    if set(parsed) != set(sources) or set(parsed) != REQUIRED_CASES:
        fail("corpus cases differ from the required exact case set")
    return parsed


def validate_case(name: str, source: bytes, expected: tuple[int, ...], gear: tuple[int, ...]) -> None:
    if expected != reference_boundaries(source, gear):
        fail(f"{name}: expected boundaries differ from scalar Gear64/FastCDC")
    previous = 0
    chunks = []
    for index, boundary in enumerate(expected):
        if boundary <= previous or boundary > len(source):
            fail(f"{name}: boundaries are not strictly increasing and bounded")
        size = boundary - previous
        if size > MAXIMUM or (index + 1 < len(expected) and size < MINIMUM):
            fail(f"{name}: chunk size {size} violates profile bounds")
        chunks.append(source[previous:boundary])
        previous = boundary
    if (not source and expected) or (source and (not expected or previous != len(source))):
        fail(f"{name}: boundaries do not cover the source exactly")
    if b"".join(chunks) != source:
        fail(f"{name}: boundary reconstruction differs from source")
    schedules = [
        [source] if source else [],
        scheduled_parts(source, itertools.cycle([4_093])),
        scheduled_parts(source, itertools.cycle([1, 7, 257, 4_093, 65_521])),
        boundary_adjacent_parts(source, expected),
    ]
    if len(source) <= MINIMUM + 1:
        schedules.append(scheduled_parts(source, itertools.cycle([1])))
    if name == "probe-byte-carry":
        schedules.append(scheduled_parts(source, itertools.repeat(1)))
    for schedule_index, parts in enumerate(schedules):
        oracle = StreamingChunker(gear)
        for part in parts:
            oracle.feed(part)
        stream_chunks, stream_ends = oracle.finish()
        if stream_ends != expected or b"".join(stream_chunks) != source:
            fail(f"{name}: partition schedule {schedule_index} moved boundaries")


def validate_named_laws(
    sources: dict[str, bytes], boundaries: dict[str, tuple[int, ...]], gear: tuple[int, ...]
) -> None:
    if boundaries["empty"] != ():
        fail("empty input must emit no chunks")
    for name in ("tiny", "min-minus-one", "min-exact"):
        if boundaries[name] != (len(sources[name]),):
            fail(f"{name}: EOF runt law moved")
    if boundaries["max-exact"] != (MAXIMUM,):
        fail("max-exact: maximum-size forced cut moved")
    if boundaries["max-plus-one"] != (MAXIMUM, MAXIMUM + 1):
        fail("max-plus-one: forced cut plus EOF runt law moved")
    carry = boundaries["probe-byte-carry"]
    carry_hash = probe_fingerprint(sources["probe-byte-carry"], carry[0], gear)
    suffix = reference_boundaries(sources["probe-byte-carry"][carry[0]:], gear)
    if carry_hash & LONG_MASK or carry[0] + suffix[0] != carry[1]:
        fail("probe-byte-carry: exclusive probe carry/reset witness moved")
    short = boundaries["short-mask-match"][0]
    if not MINIMUM < short < TARGET or probe_fingerprint(sources["short-mask-match"], short, gear) & SHORT_MASK:
        fail("short-mask-match: short-region witness moved")
    transition = boundaries["target-long-transition"][0]
    transition_hash = probe_fingerprint(sources["target-long-transition"], transition, gear)
    if transition != TARGET or transition_hash & LONG_MASK or not transition_hash & SHORT_MASK:
        fail("target-long-transition: exact mask transition witness moved")
    runt = boundaries["natural-cut-runt"]
    if len(runt) != 2 or not 0 < len(sources["natural-cut-runt"]) - runt[0] < MINIMUM:
        fail("natural-cut-runt: EOF remainder witness moved")
    oracle = StreamingChunker(gear)
    for part in scheduled_parts(sources["probe-byte-carry"], itertools.cycle([1, 4_093, 65_521])):
        before = (bytes(oracle.current), tuple(oracle.completed), oracle.fingerprint)
        oracle.feed(b"")
        if before != (bytes(oracle.current), tuple(oracle.completed), oracle.fingerprint):
            fail("empty-interleaved: empty feed changed chunker state")
        oracle.feed(part)
    _, empty_ends = oracle.finish()
    oracle.feed(b"")
    if empty_ends != carry or oracle.finish()[1] != carry:
        fail("empty-interleaved: empty feed flushed or moved a boundary")


def main() -> None:
    gear = validate_profile()
    sources = load_sources()
    expected = parse_boundaries(sources)
    for name in sorted(sources):
        validate_case(name, sources[name], expected[name], gear)
    validate_named_laws(sources, expected, gear)
    total = sum(len(value) for value in sources.values())
    print(f"CDC corpus check passed: {len(sources)} cases, {total} source bytes")


if __name__ == "__main__":
    main()

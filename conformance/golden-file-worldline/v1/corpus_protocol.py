"""Canonical admission rules for Golden File Worldline corpus tables."""

from __future__ import annotations

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parent
MAX_SOURCE_BYTES = 1_048_576
MAX_TABLE_BYTES = 1_048_576
MAX_MUTATION_VALUE_BYTES = 64
U16_MAX = (1 << 16) - 1
U64_MAX = (1 << 64) - 1
CASE_PATTERN = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*\Z")
LOWER_HEX = frozenset("0123456789abcdef")


def fail(message: str) -> None:
    raise SystemExit(f"golden corpus check failed: {message}")


def bounded_file_bytes(
    path: pathlib.Path,
    maximum: int,
    label: str,
) -> bytes:
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


def rows(
    path: pathlib.Path,
    schema: str,
    columns: list[str],
) -> list[dict[str, str]]:
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


def decoded_hex(
    value: str,
    field: str,
    maximum_bytes: int,
    allow_empty: bool = False,
) -> bytes:
    if not value and not allow_empty:
        fail(f"{field}: empty hexadecimal value")
    if len(value) > maximum_bytes * 2 or len(value) % 2 != 0:
        fail(f"{field}: hexadecimal length is invalid or unbounded")
    if any(character not in LOWER_HEX for character in value):
        fail(f"{field}: hexadecimal value is not canonical lowercase")
    return bytes.fromhex(value)


def safe_source_path(parameter: str) -> pathlib.Path:
    relative = pathlib.PurePosixPath(parameter)
    if relative.is_absolute() or any(
        part in ("", ".", "..") for part in relative.parts
    ):
        fail(f"unsafe source path: {parameter}")
    path = ROOT.joinpath(*relative.parts).resolve()
    if ROOT not in path.parents or not path.is_file():
        fail(
            "source is outside the corpus or is not a file: "
            f"{parameter}"
        )
    return path

#!/usr/bin/env python3
"""Materialize bounded deterministic seeds for every Keep fuzz target."""

from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "fuzz" / "corpus"
IDENTITIES = ROOT / "conformance" / "golden-file-worldline" / "v1" / "identities.tsv"
MAX_FIXTURE_BYTES = 1_048_576
U64_MASK = (1 << 64) - 1
CDC_SEED_SHA256 = {
    "minimum": "4fe7b59af6de3b665b67788cc2f99892ab827efae3a467342b3bb4e3bc8e5bfe",
    "short-mask-match": "9c124a59f87e94a5d85a8f46bacf04d468ed646990a1e56d5ec6a493982c106f",
    "probe-byte-carry": "d6940ea24f76d773f6a5ed3909556a74518fc902401653a254f51a6969ed941d",
    "forced-maximum": "b27a032984ea8a6bec700c3d6f63f8fcfbf8ff8ef87e972891feda4eea4aad0c",
    "random-long": "0d6c77a9edbe21f69d9c4270dca23250383805e394ad56a0c71d6f0fba181d17",
}


def fail(message: str) -> None:
    raise SystemExit(f"fuzz corpus preparation failed: {message}")


def identity_rows() -> dict[str, list[str]]:
    try:
        raw = IDENTITIES.read_bytes()
    except OSError as error:
        fail(f"cannot read identities.tsv: {error}")
    if not raw or len(raw) > MAX_FIXTURE_BYTES or b"\r" in raw or not raw.endswith(b"\n"):
        fail("identities.tsv framing or size moved")
    try:
        lines = raw.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        fail(f"identities.tsv is not UTF-8: {error}")
    expected_header = (
        "case\tsource_kind\tsource_parameter\trepetitions\tlogical_length\t"
        "canonical_text\tcanonical_binary_hex"
    )
    if len(lines) < 3 or lines[1] != expected_header:
        fail("identities.tsv header moved")
    rows: dict[str, list[str]] = {}
    for line in lines[2:]:
        fields = line.split("\t")
        if len(fields) != 7 or fields[0] in rows:
            fail("identity row is malformed or duplicated")
        rows[fields[0]] = fields
    return rows


def xorshift64(seed: int, count: int) -> bytes:
    if seed == 0 or count > MAX_FIXTURE_BYTES:
        fail("xorshift seed or count is outside its bound")
    output = bytearray(count)
    state = seed
    for index in range(count):
        state ^= (state << 13) & U64_MASK
        state ^= state >> 7
        state ^= (state << 17) & U64_MASK
        state &= U64_MASK
        output[index] = state & 0xFF
    return bytes(output)


def write_seed(target: str, name: str, content: bytes) -> None:
    if len(content) > MAX_FIXTURE_BYTES:
        fail(f"{target}/{name} exceeds the input bound")
    directory = CORPUS / target
    try:
        directory.mkdir(parents=True, exist_ok=True)
        destination = directory / name
        if destination.exists() and destination.read_bytes() == content:
            return
        destination.write_bytes(content)
    except OSError as error:
        fail(f"cannot write {target}/{name}: {error}")


def prepare_identity_seeds() -> None:
    rows = identity_rows()
    for name in ["empty", "small-text", "large-ramp"]:
        fields = rows.get(name)
        if fields is None:
            fail(f"required identity {name!r} is absent")
        write_seed("blob_id_text", name, fields[5].encode("ascii"))
        try:
            binary = bytes.fromhex(fields[6])
        except ValueError as error:
            fail(f"{name} binary identity is not hexadecimal: {error}")
        write_seed("blob_id_binary", name, binary)

    write_seed("blob_hasher", "empty", b"")
    write_seed("blob_hasher", "byte-ramp", bytes(range(256)) * 16)


def prepare_cdc_seeds() -> None:
    seeds = {
        "minimum": bytes(16_384),
        "short-mask-match": xorshift64(9, 60_000),
        "probe-byte-carry": xorshift64(0x0123_4567_89AB_CDEF, 150_000),
        "forced-maximum": bytes(262_145),
        "random-long": xorshift64(0x0123_4567_89AB_CDEF, 1_048_576),
    }
    for name, content in seeds.items():
        observed = hashlib.sha256(content).hexdigest()
        if observed != CDC_SEED_SHA256[name]:
            fail(f"CDC seed {name!r} moved from its reviewed bytes")
        write_seed("fast_cdc", name, content)


def main() -> None:
    prepare_identity_seeds()
    prepare_cdc_seeds()
    total = sum(path.stat().st_size for path in CORPUS.glob("*/*"))
    print(f"Prepared bounded fuzz seeds under {CORPUS}: {total} bytes")


if __name__ == "__main__":
    main()

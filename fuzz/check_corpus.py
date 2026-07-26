#!/usr/bin/env python3
"""Admit only a bounded, regular-file fuzz corpus as derived state."""

from __future__ import annotations

import stat
import sys
from dataclasses import dataclass
from pathlib import Path

from campaign_policy import CampaignPolicy, PolicyError, load_policy

CORPUS_DIRECTORY = Path(__file__).with_name("corpus")
TARGET_DIRECTORY = Path(__file__).with_name("fuzz_targets")


class CorpusError(ValueError):
    """The derived corpus is malformed, unsafe, or outside its bounds."""


@dataclass(frozen=True)
class CorpusStats:
    """Bounded evidence describing the admitted derived corpus."""

    files: int
    bytes: int


def target_names() -> set[str]:
    """Return the checked-in fuzz target names."""
    names = {path.stem for path in TARGET_DIRECTORY.glob("*.rs")}
    if not names:
        raise CorpusError("the checked-in fuzz target set is empty")
    return names


def admit_file(
    path: Path,
    policy: CampaignPolicy,
    files: int,
    bytes_total: int,
) -> tuple[int, int]:
    """Admit one regular corpus file without following links."""
    try:
        metadata = path.stat(follow_symlinks=False)
    except OSError as error:
        raise CorpusError(f"cannot inspect corpus entry {path}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise CorpusError(f"corpus entry is not a regular file: {path}")
    size = metadata.st_size
    if size > policy.max_input_bytes:
        raise CorpusError(f"corpus entry exceeds the input bound: {path}")
    next_files = files + 1
    next_bytes = bytes_total + size
    if next_files > policy.corpus_max_files:
        raise CorpusError("corpus file count exceeds its bound")
    if next_bytes > policy.corpus_max_bytes:
        raise CorpusError("corpus byte count exceeds its bound")
    return next_files, next_bytes


def audit_corpus(
    root: Path,
    policy: CampaignPolicy,
) -> CorpusStats:
    """Validate the entire derived corpus tree without following links."""
    try:
        root_metadata = root.stat(follow_symlinks=False)
    except FileNotFoundError:
        return CorpusStats(files=0, bytes=0)
    except OSError as error:
        raise CorpusError(f"cannot inspect corpus root {root}: {error}") from error
    if not stat.S_ISDIR(root_metadata.st_mode):
        raise CorpusError(f"corpus root is not a directory: {root}")

    expected_targets = target_names()
    files = 0
    bytes_total = 0
    for target_path in sorted(root.iterdir()):
        try:
            target_metadata = target_path.stat(follow_symlinks=False)
        except OSError as error:
            raise CorpusError(
                f"cannot inspect corpus target {target_path}: {error}"
            ) from error
        if not stat.S_ISDIR(target_metadata.st_mode):
            raise CorpusError(f"unexpected corpus target: {target_path}")
        if target_path.name not in expected_targets:
            raise CorpusError(f"unexpected corpus target: {target_path}")
        for path in sorted(target_path.iterdir()):
            files, bytes_total = admit_file(
                path,
                policy,
                files,
                bytes_total,
            )
    return CorpusStats(files=files, bytes=bytes_total)


def main() -> int:
    """Report admitted corpus bounds or refuse the derived state."""
    try:
        policy = load_policy()
        stats = audit_corpus(CORPUS_DIRECTORY, policy)
    except (CorpusError, PolicyError) as error:
        print(f"fuzz corpus refused: {error}", file=sys.stderr)
        return 1
    print(f"Admitted fuzz corpus: {stats.files} files, {stats.bytes} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())

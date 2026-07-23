"""Independent scalar FastCDC oracle owned by the v1 conformance corpus."""

from __future__ import annotations

from collections.abc import Iterable, Iterator
import itertools

MINIMUM, TARGET, MAXIMUM = 16_384, 65_536, 262_144
NORMALIZATION, STATE_WIDTH, SEED = 2, 64, 0
SHORT_MASK = 0x0000_D907_0753_7000
LONG_MASK = 0x0000_D903_1353_0000
U64_MASK = (1 << 64) - 1


def reference_boundaries(source: bytes, gear: tuple[int, ...]) -> tuple[int, ...]:
    """Return absolute exclusive ends from the one-shot scalar algorithm."""
    ends: list[int] = []
    start = 0
    while start < len(source):
        remaining = len(source) - start
        if remaining <= MINIMUM:
            start += remaining
            ends.append(start)
            continue
        limit = min(remaining, MAXIMUM)
        center = min(TARGET, limit)
        fingerprint = SEED
        cut = limit
        for position in range(MINIMUM, center):
            fingerprint = ((fingerprint << 1) + gear[source[start + position]]) & U64_MASK
            if fingerprint & SHORT_MASK == 0:
                cut = position
                break
        else:
            for position in range(center, limit):
                fingerprint = ((fingerprint << 1) + gear[source[start + position]]) & U64_MASK
                if fingerprint & LONG_MASK == 0:
                    cut = position
                    break
        start += cut
        ends.append(start)
    return tuple(ends)


class StreamingChunker:
    """Scalar oracle whose feed partitioning cannot affect its state machine."""

    def __init__(self, gear: tuple[int, ...]) -> None:
        self.gear = gear
        self.current = bytearray()
        self.completed: list[bytes] = []
        self.fingerprint = SEED

    def feed(self, part: bytes) -> None:
        """Advance state without treating an empty part as EOF."""
        for value in part:
            self._feed_byte(value)

    def _feed_byte(self, value: int) -> None:
        position = len(self.current)
        if position < MINIMUM:
            self.current.append(value)
            return
        self.fingerprint = ((self.fingerprint << 1) + self.gear[value]) & U64_MASK
        mask = SHORT_MASK if position < TARGET else LONG_MASK
        if self.fingerprint & mask == 0:
            self._emit()
            self.current.append(value)
            return
        self.current.append(value)
        if len(self.current) == MAXIMUM:
            self._emit()

    def _emit(self) -> None:
        if not self.current:
            raise ValueError("streaming oracle cannot emit an empty chunk")
        self.completed.append(bytes(self.current))
        self.current.clear()
        self.fingerprint = SEED

    def finish(self) -> tuple[list[bytes], tuple[int, ...]]:
        """Declare EOF and return chunks plus their absolute exclusive ends."""
        if self.current:
            self._emit()
        total = 0
        ends = []
        for chunk in self.completed:
            total += len(chunk)
            ends.append(total)
        return self.completed, tuple(ends)


def scheduled_parts(source: bytes, sizes: Iterable[int]) -> Iterator[bytes]:
    """Partition source bytes according to a positive repeating size schedule."""
    offset = 0
    iterator = iter(sizes)
    while offset < len(source):
        size = next(iterator)
        if size <= 0:
            raise ValueError("partition sizes must be positive")
        end = min(len(source), offset + size)
        yield source[offset:end]
        offset = end


def boundary_adjacent_parts(source: bytes, ends: tuple[int, ...]) -> list[bytes]:
    """Partition immediately before, at, and after every expected boundary."""
    points = {0, len(source)}
    for boundary in ends:
        points.update({max(0, boundary - 1), boundary, min(len(source), boundary + 1)})
    ordered = sorted(points)
    return [source[left:right] for left, right in itertools.pairwise(ordered) if left < right]


def probe_fingerprint(source: bytes, position: int, gear: tuple[int, ...]) -> int:
    """Return the reset fingerprint after including probe byte at position."""
    fingerprint = SEED
    for offset in range(MINIMUM, position + 1):
        fingerprint = ((fingerprint << 1) + gear[source[offset]]) & U64_MASK
    return fingerprint

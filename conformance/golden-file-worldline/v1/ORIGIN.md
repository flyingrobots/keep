# Fixture Origin and License

Every input in this directory was created specifically for Keep and is licensed
under the repository's Apache License 2.0.

- `inputs/small-text.txt` is the exact ASCII byte string documented by
  ADR-0001.
- `inputs/state-a.txt` is a short project-authored sequence of LF-terminated
  lines.
- `inputs/state-b.txt` is state A with the exact ASCII bytes `INSERTED\n`
  inserted after byte offset 6.
- `byte-ramp-v1` is the mathematical sequence of all unsigned octets from 0
  through 255, repeated as declared by the manifest.
- the empty source contains no bytes.

The source kind, repetition count, exact length, canonical identity, and
canonical binary identity are declared in `identities.tsv`.

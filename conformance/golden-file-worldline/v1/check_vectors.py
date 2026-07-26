#!/usr/bin/env python3
"""Validate Golden File Worldline v1 without importing Keep."""

from __future__ import annotations

import subprocess

from capability_oracle import check_capabilities
from corpus_protocol import fail
from identity_oracle import check_identities, check_invalid_text
from scenario_oracle import check_mutations, check_steps


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
        fail(
            "required file or b3sum executable not found: "
            f"{error.filename}"
        )
    except OSError as error:
        fail(f"operating-system failure: {error}")

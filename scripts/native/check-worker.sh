#!/usr/bin/env bash
# Compiler-only child: all writable locations are on the bounded native images.
set -euo pipefail
readonly source_dir="$1"
readonly build_dir="$2"
readonly scratch_dir="$3"
readonly log_path="$4"
readonly sandbox_profile="$5"

export CARGO_HOME="$build_dir/cargo-home"
export CARGO_TARGET_DIR="$build_dir/target"
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
export CARGO_NET_OFFLINE=true
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export TMPDIR="$scratch_dir/"
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
cd "$source_dir"

# The monitor caps aggregate RSS; these inherited ceilings also bound each
# compiler process independently. No test executable is launched by this tool.
(
  ulimit -t 900
  ulimit -v 4194304
  sandbox-exec -f "$sandbox_profile" \
    cargo check --lib --all-features --locked --offline
) 2>&1 | (
  # This limit applies to the log writer, not compiler artifact sizes.
  ulimit -f 4096
  cat > "$log_path"
)

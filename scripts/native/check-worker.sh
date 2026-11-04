#!/usr/bin/env bash
# Compile or test child: all writable locations are on the bounded native images.
set -euo pipefail
readonly source_dir="$1"
readonly build_dir="$2"
readonly scratch_dir="$3"
readonly log_path="$4"
readonly sandbox_profile="$5"
readonly check_scope="$6"
readonly data_variant="$7"

export CARGO_HOME="$build_dir/cargo-home"
export CARGO_TARGET_DIR="$build_dir/target"
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
export CARGO_NET_OFFLINE=true
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export TMPDIR="$scratch_dir/"
export CARGO_TARGET_TMPDIR="$scratch_dir/test"
export KEEP_TEST_SCRATCH_ROOT="${scratch_dir%/}/test"
export KEEP_TEST_EXPECT_APFS_VARIANT="$data_variant"
unset RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
cd "$source_dir"

# A package-manager cargo can precede rustup on PATH. Evidence must come from the
# repository's pinned toolchain, including its clippy.
readonly toolchain="$(awk -F'"' '/^channel/ {print $2}' rust-toolchain.toml)"
PATH="$(dirname "$(rustup which --toolchain "$toolchain" cargo)"):$PATH"
export PATH
[[ "$(rustc --version)" == "rustc $toolchain "* ]]

if [[ "$check_scope" == --run-tests ]]; then
  cargo_command=(cargo test --all-features --locked --offline --no-fail-fast)
elif [[ "$check_scope" == --crash-matrix ]]; then
  cargo_command=(cargo xtask durability-crash-matrix)
elif [[ "$check_scope" == --clippy ]]; then
  cargo_command=(cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings)
else
  cargo_command=(cargo check "$check_scope" --all-features --locked --offline)
fi

# The parent monitor caps aggregate RSS. The inherited CPU ceiling also bounds
# each compiler or test process.
(
  ulimit -t 900
  rustc --version
  sandbox-exec -f "$sandbox_profile" \
    "${cargo_command[@]}"
) 2>&1 | (
  # This limit applies to the log writer, not compiler artifact sizes.
  ulimit -f 4096
  cat > "$log_path"
)

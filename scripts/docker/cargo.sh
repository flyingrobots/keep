#!/usr/bin/env bash
# Reuse the existing Keep worker; copy source, never host build artifacts.
set -euo pipefail

readonly source_dir="$(cd -- "${1:?usage: cargo.sh CHECKOUT CARGO_ARGUMENTS...}" && pwd)"
shift
[[ $# -gt 0 && -f "$source_dir/Cargo.toml" ]] || exit 2
readonly container="${KEEP_DOCKER_CONTAINER:-keep-validation}"
readonly script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly free_kib="$(df -Pk "$source_dir" | awk 'END {print $4}')"
if [[ ! "$free_kib" =~ ^[0-9]+$ ]] || (( free_kib < 52428800 )); then
  printf 'Refusing validation: the checkout filesystem needs 50 GiB free.\n' >&2
  exit 1
fi
if [[ "$(docker inspect --format '{{.State.Running}}' "$container")" != true ]]; then
  printf 'Start the existing %s worker; this command never creates images or workers.\n' "$container" >&2
  exit 1
fi

(
  cd "$source_dir"
  find . \( -path './.git' -o -path './target' -o -path './fuzz/target' \
    -o -path './xtask/target' -o -name node_modules \) -prune \
    -o \( -type f -o -type l \) ! -name .DS_Store -print0
) |
  COPYFILE_DISABLE=1 tar -C "$source_dir" --null --no-recursion -T - -cf - |
  docker exec -i "$container" bash -c "$(cat "$script_dir/cargo-worker.sh")" keep-cargo "$@"

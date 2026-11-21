#!/usr/bin/env bash
# Owns one disposable source copy and one bounded Cargo cache in the worker.
set -euo pipefail

exec 9>/var/lock/keep-validation.lock
flock -n 9 || { printf 'Keep validation is already running.\n' >&2; exit 1; }
for argument in "$@"; do
  case "$argument" in
    --target-dir*|--config*)
      printf 'Cache/config overrides are forbidden by the bounded worker.\n' >&2
      exit 2 ;;
  esac
done

readonly image=/var/cache/keep-validation.ext4
readonly cache=/keep-validation-cache
readonly image_bytes=21474836480
mkdir -p "$cache"
if [[ ! -e "$image" ]]; then
  truncate -s "$image_bytes" "$image.new"
  mkfs.ext4 -q -m 0 -L KEEP_CARGO_CACHE "$image.new"
  mv -- "$image.new" "$image"
fi
[[ "$(stat -c %s "$image")" == "$image_bytes" ]]
[[ "$(blkid -s LABEL -o value "$image")" == KEEP_CARGO_CACHE ]]
if ! mountpoint -q "$cache"; then
  mount -o loop,nodev,nosuid "$image" "$cache"
fi
readonly device="$(findmnt -n -o SOURCE --mountpoint "$cache")"
[[ "$(findmnt -n -o FSTYPE --mountpoint "$cache")" == ext4 ]]
[[ "$(losetup -n -O BACK-FILE "$device")" == "$image" ]]

# A changed snapshot gets fresh mtimes so older parent sources cannot reuse
# newer candidate binaries. Unchanged snapshots retain Cargo's warm cache.
cat > "$cache/source.tar.new"
readonly snapshot="$(sha256sum "$cache/source.tar.new" | cut -d ' ' -f 1)"
previous_snapshot=''
if [[ -f "$cache/source.sha256" ]]; then
  previous_snapshot="$(cat "$cache/source.sha256")"
fi
if [[ "$snapshot" != "$previous_snapshot" || ! -d "$cache/source" ]]; then
  # Only this disposable copy is replaced; host sources and receipts stay outside.
  rm -rf --one-file-system -- "$cache/source"
  mkdir "$cache/source"
  tar -xmf "$cache/source.tar.new" -C "$cache/source"
  printf '%s\n' "$snapshot" > "$cache/source.sha256"
fi
rm -- "$cache/source.tar.new"
cd "$cache/source"
export CARGO_TARGET_DIR="$cache/target"
export CARGO_INCREMENTAL=0
export CARGO_NET_OFFLINE=true
export CARGO_BUILD_JOBS=2
cargo "$@"

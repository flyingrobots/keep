#!/usr/bin/env bash
set -euo pipefail

readonly install_root="${1:?usage: install_documentation_tools.sh INSTALL_ROOT}"
readonly actionlint_version="1.7.12"
readonly lychee_version="0.21.0"
readonly script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly npm_manifest_dir="$script_dir/documentation-tools"

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
  printf 'documentation CI tools require Linux x86_64\n' >&2
  exit 1
fi

readonly binary_dir="$install_root/bin"
readonly npm_dir="$install_root/npm"
readonly scratch_dir="$install_root/downloads"
mkdir -p "$binary_dir" "$npm_dir" "$scratch_dir"

download_and_verify() {
  local url="$1"
  local expected_sha256="$2"
  local output="$3"
  local observed_sha256

  curl \
    --fail \
    --location \
    --proto '=https' \
    --tlsv1.2 \
    --retry 3 \
    --retry-all-errors \
    --connect-timeout 20 \
    --max-time 120 \
    --output "$output" \
    "$url"
  observed_sha256="$(sha256sum "$output")"
  observed_sha256="${observed_sha256%% *}"
  if [[ "$observed_sha256" != "$expected_sha256" ]]; then
    printf 'SHA-256 mismatch for %s\n' "$output" >&2
    exit 1
  fi
}

readonly actionlint_archive="$scratch_dir/actionlint.tar.gz"
download_and_verify \
  "https://github.com/rhysd/actionlint/releases/download/v${actionlint_version}/actionlint_${actionlint_version}_linux_amd64.tar.gz" \
  "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8" \
  "$actionlint_archive"
mkdir -p "$scratch_dir/actionlint"
tar --extract --gzip --file "$actionlint_archive" \
  --directory "$scratch_dir/actionlint"
install --mode 0755 "$scratch_dir/actionlint/actionlint" \
  "$binary_dir/actionlint"

readonly lychee_archive="$scratch_dir/lychee.tar.gz"
download_and_verify \
  "https://github.com/lycheeverse/lychee/releases/download/lychee-v${lychee_version}/lychee-x86_64-unknown-linux-gnu.tar.gz" \
  "a06547250f10021dcafc6ed5bb20fca75835b65711745b63cfdda34c29ff6a73" \
  "$lychee_archive"
mkdir -p "$scratch_dir/lychee"
tar --extract --gzip --file "$lychee_archive" \
  --directory "$scratch_dir/lychee"
install --mode 0755 "$scratch_dir/lychee/lychee" "$binary_dir/lychee"

install --mode 0644 "$npm_manifest_dir/package.json" "$npm_dir/package.json"
install --mode 0644 \
  "$npm_manifest_dir/package-lock.json" \
  "$npm_dir/package-lock.json"
npm ci \
  --prefix "$npm_dir" \
  --ignore-scripts \
  --no-audit \
  --no-fund

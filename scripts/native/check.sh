#!/usr/bin/env bash
# Bounded native compilation. Invoke under the workstation resource locks.
set -euo pipefail

readonly source_dir="$(cd -- "${1:?usage: check.sh CHECKOUT}" && pwd)"
readonly native_root="${KEEP_NATIVE_ROOT:-$HOME/Library/Caches/keep-native-validation}"
readonly build_dir="$native_root/build"
readonly scratch_dir="$native_root/data/tmp"
readonly logs_dir="$native_root/logs"
readonly profile="$native_root/compiler.sb"
readonly log_path="$logs_dir/$(date -u +%Y%m%dT%H%M%SZ)-check.log"
readonly worker="$source_dir/scripts/native/check-worker.sh"
readonly minimum_free_kib=52428800

[[ "$(uname -s)" == Darwin && -f "$source_dir/Cargo.toml" ]]
[[ -d "$build_dir/target" && -d "$scratch_dir" && -d "$logs_dir" ]]
# Setup is deliberate and separate. Refuse an absent quota filesystem instead
# of compiling into an ordinary directory left behind by an unmount.
[[ "$(df -Pk "$build_dir" | awk 'END {print $6}')" == "$build_dir" ]]
[[ "$(df -Pk "$scratch_dir" | awk 'END {print $6}')" == "$native_root/data" ]]
[[ "$(df -Pk "$build_dir" | awk 'END {print $2}')" -le 1572864 ]]
[[ "$(df -Pk "$scratch_dir" | awk 'END {print $2}')" -le 262144 ]]
[[ "$(df -Pk "$source_dir" | awk 'END {print $4}')" -ge "$minimum_free_kib" ]]
[[ "$(du -sk "$logs_dir" | awk '{print $1}')" -lt 20480 ]]

cat > "$profile" <<PROFILE
(version 1)
(allow default)
(deny network*)
(deny file-write*)
(allow file-write*
  (subpath "$build_dir/target")
  (subpath "$build_dir/cargo-home")
  (subpath "$scratch_dir")
  (literal "/dev/null"))
PROFILE

task_pid=''
stop_child() {
  if [[ -n "$task_pid" ]]; then
    kill -KILL -- "-$task_pid" 2>/dev/null || true
    wait "$task_pid" 2>/dev/null || true
  fi
}
trap stop_child EXIT INT TERM

# setsid gives the compiler tree one owned process group. This runner admits
# compilation only, excluding test harnesses that create their own groups.
/usr/bin/perl -MPOSIX=setsid -e \
  'setsid() >= 0 or die "setsid failed"; exec @ARGV or die "exec failed";' \
  /bin/bash "$worker" "$source_dir" "$build_dir" "$scratch_dir" \
  "$log_path" "$profile" &
task_pid=$!
readonly task_started=$SECONDS
failure=''
while kill -0 "$task_pid" 2>/dev/null; do
  if ! process_snapshot="$(ps -axo pgid=,rss=)"; then
    failure='process monitor failed'
    break
  fi
  task_rss="$(awk -v group="$task_pid" '$1 == group {sum += $2} END {print sum+0}' <<< "$process_snapshot")"
  if ! free_kib="$(df -Pk "$source_dir" | awk 'END {print $4}')"; then
    failure='disk monitor failed'
    break
  fi
  if (( task_rss > 6291456 || SECONDS - task_started > 1200 || free_kib < minimum_free_kib )); then
    failure='memory, timeout, or free-space guard refused'
    break
  fi
  sleep 1
done

if [[ -n "$failure" ]]; then
  printf '%s\n' "$failure" >&2
  exit 1
fi
result=0
wait "$task_pid" || result=$?
stop_child
task_pid=''
tail -60 "$log_path"
printf 'Native compilation log: %s\n' "$log_path"
exit "$result"

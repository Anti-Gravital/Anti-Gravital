#!/usr/bin/env bash
# Alpha installer channel verification with a local fake release directory.
#
# Uses file:// URLs only: no network access and no privileged install. It
# exercises the real download -> checksum verify -> install path of install.sh
# and the fail-closed cases (mismatch, missing manifest, missing binary, missing
# source, unknown channel). Called by tools/check-installers.sh.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

release="$work/release"
bin_dir="$work/bin"
mkdir -p "$release" "$bin_dir"

target="x86_64-unknown-linux-gnu"
binary_name="ag-$target"
printf 'fake ag binary for alpha channel test\n' >"$release/$binary_name"
binary_hash="$(sha256sum "$release/$binary_name" | awk '{print $1}')"
printf '%s  %s\n' "$binary_hash" "$binary_name" >"$release/checksums.txt"

run_alpha() {
    env -u AG_INSTALLER_CHECKSUMS \
        AG_INSTALLER_CHANNEL=alpha \
        AG_INSTALLER_BASE_URL="file://$release" \
        AG_INSTALLER_TARGET="$target" \
        AG_INSTALLER_BIN_DIR="$bin_dir" \
        bash install.sh
}

run_alpha_with_manifest() {
    env -u AG_INSTALLER_CHECKSUMS \
        AG_INSTALLER_CHANNEL=alpha \
        AG_INSTALLER_BASE_URL="file://$release" \
        AG_INSTALLER_TARGET="$target" \
        AG_INSTALLER_BIN_DIR="$bin_dir" \
        AG_INSTALLER_BINARY_CHECKSUMS="$1" \
        bash install.sh
}

# 1. Valid manifest: verify, then install.
run_alpha >"$work/ok.log" 2>&1
grep -F 'Checksum verified for ag-x86_64-unknown-linux-gnu' "$work/ok.log" >/dev/null
grep -F 'Installation complete.' "$work/ok.log" >/dev/null
test -x "$bin_dir/ag"
printf -- '--- alpha valid manifest (expected pass) ---\n'
cat "$work/ok.log"

# 2. Mismatched checksum: abort before install.
printf '%064d  %s\n' 0 "$binary_name" >"$release/checksums.txt"
rm -f "$bin_dir/ag"
if run_alpha >"$work/bad.log" 2>&1; then
    printf 'alpha installer accepted a mismatched checksum.\n' >&2
    exit 1
fi
grep -F 'Checksum mismatch for ag-x86_64-unknown-linux-gnu' "$work/bad.log" >/dev/null
test ! -e "$bin_dir/ag"
printf -- '--- alpha mismatched checksum (expected failure) ---\n'
cat "$work/bad.log"

# 3. Missing manifest: abort before install.
rm -f "$release/checksums.txt"
if run_alpha >"$work/missing.log" 2>&1; then
    printf 'alpha installer accepted a missing manifest.\n' >&2
    exit 1
fi
grep -F 'Could not download the checksum manifest' "$work/missing.log" >/dev/null
test ! -e "$bin_dir/ag"
printf -- '--- alpha missing manifest (expected failure) ---\n'
cat "$work/missing.log"

# 4. Missing binary: abort before install.
printf '%s  %s\n' "$binary_hash" "$binary_name" >"$release/checksums.txt"
rm -f "$release/$binary_name"
if run_alpha >"$work/nobin.log" 2>&1; then
    printf 'alpha installer accepted a missing binary.\n' >&2
    exit 1
fi
grep -F 'Could not download the binary' "$work/nobin.log" >/dev/null
test ! -e "$bin_dir/ag"
printf -- '--- alpha missing binary (expected failure) ---\n'
cat "$work/nobin.log"

# 5. No base URL and no release tag: fail closed with guidance.
if env -u AG_INSTALLER_CHECKSUMS -u AG_INSTALLER_BASE_URL -u AG_INSTALLER_RELEASE_TAG \
        AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_TARGET="$target" \
        AG_INSTALLER_BIN_DIR="$bin_dir" bash install.sh >"$work/nourl.log" 2>&1; then
    printf 'alpha installer accepted a missing source.\n' >&2
    exit 1
fi
grep -F 'requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG' "$work/nourl.log" >/dev/null
printf -- '--- alpha missing source (expected failure) ---\n'
cat "$work/nourl.log"

# 6. Unknown channel: reject before any action.
if env -u AG_INSTALLER_CHECKSUMS AG_INSTALLER_CHANNEL=bogus \
        bash install.sh >"$work/bogus.log" 2>&1; then
    printf 'installer accepted an unknown channel.\n' >&2
    exit 1
fi
grep -F 'Unknown AG_INSTALLER_CHANNEL' "$work/bogus.log" >/dev/null
printf -- '--- unknown channel (expected failure) ---\n'
cat "$work/bogus.log"

# 7. Out-of-band binary manifest: the release copy is absent, so a match can
# only come from the locally supplied manifest.
printf 'fake ag binary for alpha channel test\n' >"$release/$binary_name"
new_hash="$(sha256sum "$release/$binary_name" | awk '{print $1}')"
printf '%s  %s\n' "$new_hash" "$binary_name" >"$work/local.sha256"
rm -f "$release/checksums.txt" "$bin_dir/ag"
run_alpha_with_manifest "$work/local.sha256" >"$work/local.log" 2>&1
grep -F 'using local binary checksum manifest' "$work/local.log" >/dev/null
grep -F 'Checksum verified for ag-x86_64-unknown-linux-gnu' "$work/local.log" >/dev/null
test -x "$bin_dir/ag"
printf -- '--- alpha out-of-band manifest (expected pass) ---\n'
cat "$work/local.log"

# 8. Missing local binary manifest: fail closed.
rm -f "$bin_dir/ag"
if run_alpha_with_manifest "$work/missing.sha256" >"$work/local-missing.log" 2>&1; then
    printf 'alpha installer accepted a missing out-of-band manifest.\n' >&2
    exit 1
fi
grep -F 'Binary checksum manifest not found' "$work/local-missing.log" >/dev/null
test ! -e "$bin_dir/ag"
printf -- '--- alpha missing out-of-band manifest (expected failure) ---\n'
cat "$work/local-missing.log"

printf 'Alpha installer checks passed.\n'

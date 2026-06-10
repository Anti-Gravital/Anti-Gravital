#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

bash -n install.sh

for installer_file in install.sh install.ps1 checksums/installers.sha256; do
    eol="$(git check-attr eol -- "$installer_file" | awk '{print $3}')"
    if [[ "$eol" != "lf" ]]; then
        printf '%s must be checked out with LF line endings.\n' "$installer_file" >&2
        exit 1
    fi
done

workspace_msrv="$(sed -n 's/^rust-version = "\([0-9][0-9.]*\)"/\1/p' Cargo.toml | head -n 1)"
bash_msrv="$(sed -n 's/^REQUIRED_RUST="\([0-9][0-9.]*\)"/\1/p' install.sh)"
powershell_msrv="$(sed -n 's/^\$RequiredRust = \[version\]"\([0-9][0-9.]*\)"/\1/p' install.ps1)"

normalize_version() {
    local version=$1 dots
    dots=${version//[^.]/}
    case ${#dots} in
        0) printf '%s.0.0\n' "$version" ;;
        1) printf '%s.0\n' "$version" ;;
        *) printf '%s\n' "$version" ;;
    esac
}

expected="$(normalize_version "$workspace_msrv")"
test "$(normalize_version "$bash_msrv")" = "$expected"
test "$(normalize_version "$powershell_msrv")" = "$expected"

sha256sum --check checksums/installers.sha256

fake_bin="$(mktemp -d)"
trap 'rm -rf "$fake_bin"' EXIT

cat >"$fake_bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*"
EOF
cat >"$fake_bin/rustc" <<'EOF'
#!/usr/bin/env bash
printf 'rustc %s (test fixture)\n' "${FAKE_RUST_VERSION:?}"
EOF
chmod +x "$fake_bin/cargo" "$fake_bin/rustc"

if PATH="$fake_bin:$PATH" FAKE_RUST_VERSION=1.78.0 bash install.sh >"$fake_bin/old.log" 2>&1; then
    printf 'install.sh accepted Rust below the workspace MSRV.\n' >&2
    exit 1
fi
grep -F "Rust ${expected} or newer is required" "$fake_bin/old.log" >/dev/null
if grep -F 'cargo ' "$fake_bin/old.log" >/dev/null; then
    printf 'install.sh invoked Cargo after rejecting an old toolchain.\n' >&2
    exit 1
fi

PATH="$fake_bin:$PATH" FAKE_RUST_VERSION="$expected" bash install.sh >"$fake_bin/current.log" 2>&1
grep -F 'cargo build --workspace --release' "$fake_bin/current.log" >/dev/null
grep -F 'cargo install --path crates/ag-cli --locked' "$fake_bin/current.log" >/dev/null
grep -F 'Installation complete.' "$fake_bin/current.log" >/dev/null

printf 'Installer checks passed (MSRV %s).\n' "$expected"

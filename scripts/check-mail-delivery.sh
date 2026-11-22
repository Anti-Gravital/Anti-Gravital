#!/usr/bin/env bash
# Opt-in real-mailbox delivery gate checker for ag-mail (issue #178).
#
# This checker NEVER sends mail and NEVER proves delivery by itself. It reads a
# filled results template whose values come from a receiving MTA's
# Authentication-Results header and can only report PASS when that evidence is
# present, opted in and internally consistent.
#
# Outcomes:
#   PASS  opt-in, reachable sink or port 25, and valid recorded evidence.
#   SKIP  not opted in, no egress, no sink, missing identities or no evidence.
#         A SKIP is never a pass; read the RESULT line, not only the exit code.
#   FAIL  evidence exists but is rejected: dkim=fail/error, missing tags,
#         identity mismatch, unexpected algorithm or a local "verifier" source.
#
# Exit status is 0 for PASS and SKIP, 1 for FAIL. Default CI does not call this
# script. See docs/security/REAL_MAILBOX_VERIFICATION.md and
# docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

report() { printf '%s\n' "$*"; }
skip() { report "RESULT: SKIP reason=$1 detail=$2"; exit 0; }
fail() { report "RESULT: FAIL reason=$1 detail=$2"; exit 1; }

# --- explicit opt-in ---------------------------------------------------------
opt_in="${AG_MAIL_DELIVERY_OPT_IN:-}"
[ "$opt_in" = "1" ] || skip not-opted-in "set AG_MAIL_DELIVERY_OPT_IN=1 to enable the delivery gate check"

scope="${AG_MAIL_DELIVERY_SCOPE:-real-mailbox}"
case "$scope" in
    wire|real-mailbox) ;;
    *) fail invalid-scope "AG_MAIL_DELIVERY_SCOPE must be wire or real-mailbox" ;;
esac

results="${AG_MAIL_DELIVERY_RESULTS:-docs/security/REAL_MAILBOX_RESULTS.md}"
report "tool.bash=$BASH_VERSION"
report "scope=$scope"
report "results.file=$results"

# --- connectivity prerequisites ---------------------------------------------
# A probe target is required for the real-mailbox scope; there is deliberately
# no default host, so the maintainer states which MX they are testing.
probe_host=""
probe_port=""
if [ "$scope" = "real-mailbox" ]; then
    probe="${AG_MAIL_DELIVERY_PROBE_TARGET:-}"
    [ -n "$probe" ] || skip probe-target-not-configured "set AG_MAIL_DELIVERY_PROBE_TARGET=host:25"
    probe_host="${probe%:*}"
    probe_port="${probe##*:}"
    case "$probe_host" in
        ''|*[!A-Za-z0-9.-]*) fail invalid-probe-target "host must contain only letters, digits, dots and hyphens" ;;
    esac
    case "$probe_port" in
        ''|*[!0-9]*) fail invalid-probe-target "port must be numeric" ;;
    esac
    check_host="$probe_host"
    check_port="$probe_port"
else
    check_host="${AG_MAIL_SINK_HOST:-127.0.0.1}"
    check_port="${AG_MAIL_SINK_PORT:-1025}"
    case "$check_host" in
        ''|*[!A-Za-z0-9.:-]*) fail invalid-sink-host "sink host contains unexpected characters" ;;
    esac
    case "$check_port" in
        ''|*[!0-9]*) fail invalid-sink-port "sink port must be numeric" ;;
    esac
fi

if ! command -v timeout >/dev/null 2>&1; then
    skip egress-check-unavailable "timeout is required for a bounded TCP probe; egress cannot be verified"
fi
if ! timeout 5 bash -c 'exec 3<>/dev/tcp/"$1"/"$2"' _ "$check_host" "$check_port" \
    >/dev/null 2>&1; then
    if [ "$scope" = "real-mailbox" ]; then
        skip port-25-egress-unavailable "TCP connect to $check_host:$check_port failed or timed out"
    else
        skip sink-unavailable "TCP connect to local sink $check_host:$check_port failed or timed out"
    fi
fi
report "connect=$check_host:$check_port reachable"

# --- identities (credentials) for the live scope -----------------------------
to_addr="${AG_MAIL_DELIVERY_TO:-}"
domain="${AG_MAIL_DELIVERY_DOMAIN:-}"
selector="${AG_MAIL_DELIVERY_SELECTOR:-}"
if [ "$scope" = "real-mailbox" ]; then
    if [ -z "$to_addr" ] || [ -z "$domain" ] || [ -z "$selector" ]; then
        skip credentials-unavailable "AG_MAIL_DELIVERY_TO, AG_MAIL_DELIVERY_DOMAIN and AG_MAIL_DELIVERY_SELECTOR are all required"
    fi
    report "identity.to=$to_addr"
    report "identity.domain=$domain"
    report "identity.selector=$selector"
fi

# --- read the evidence file --------------------------------------------------
[ -f "$results" ] || skip no-evidence "results file not found; copy REAL_MAILBOX_RESULTS_TEMPLATE.md and fill it"

value_of() {
    sed -n "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*//p" "$results" | head -n 1
}

date_utc="$(value_of delivery.date_utc)"
commit_sha="$(value_of delivery.commit_sha)"
recipient_address="$(value_of delivery.recipient_address)"
signing_domain="$(value_of delivery.signing_domain)"
result="$(value_of auth_results.dkim_result)"
receiver="$(value_of auth_results.receiver)"
raw_ar="$(value_of auth_results.raw)"
header_d="$(value_of auth_results.header_d)"
header_s="$(value_of auth_results.header_s)"
header_a="$(value_of auth_results.header_a)"
header_b="$(value_of auth_results.header_b)"
sig_raw="$(value_of dkim_signature.raw)"
sig_d="$(value_of dkim_signature.d)"
sig_s="$(value_of dkim_signature.s)"
sig_a="$(value_of dkim_signature.a)"
sig_b="$(value_of dkim_signature.b)"
local_verifier="$(value_of evidence.local_verifier_used)"

# An empty template is not evidence. At least one verdict and one signature
# value must be present before the checker will evaluate the content.
if [ -z "$result" ] && [ -z "$sig_b" ] && [ -z "$raw_ar" ]; then
    skip no-evidence "results file is empty or still the unfilled template"
fi

# Evidence claiming a local verifier as its source is not receiver evidence.
if printf '%s' "$local_verifier" | grep -qi '^true$'; then
    fail local-verifier-source "a local dkim.verify() result is not an Authentication-Results header"
fi

# --- wire scope: DKIM tags on the captured bytes only ------------------------
if [ "$scope" = "wire" ]; then
    missing=""
    for pair in "dkim_signature.d:$sig_d" "dkim_signature.s:$sig_s" \
                "dkim_signature.a:$sig_a" "dkim_signature.b:$sig_b"; do
        key="${pair%%:*}"
        val="${pair#*:}"
        [ -n "$val" ] || missing="$missing $key"
    done
    [ -z "$missing" ] || fail incomplete-evidence "missing signature tags:$missing"
    case "$sig_a" in
        ed25519-sha256|rsa-sha256) ;;
        *) fail unexpected-algorithm "dkim_signature.a=$sig_a is not ed25519-sha256 or rsa-sha256" ;;
    esac
    if [ -n "$header_a" ] && [ "$header_a" != "$sig_a" ]; then
        fail algorithm-mismatch "header.a=$header_a does not match dkim_signature.a=$sig_a"
    fi
    report "wire.signature=d=$sig_d s=$sig_s a=$sig_a"
    report "gate.real_mailbox=NOT_SATISFIED"
    report "RESULT: PASS scope=wire reason=local-sink-signature-tags-present"
    exit 0
fi

# --- real-mailbox scope ------------------------------------------------------
missing=""
for pair in "delivery.date_utc:$date_utc" "delivery.commit_sha:$commit_sha" \
            "delivery.recipient_address:$recipient_address" \
            "delivery.signing_domain:$signing_domain" \
            "auth_results.dkim_result:$result" "auth_results.receiver:$receiver" \
            "auth_results.raw:$raw_ar" "auth_results.header_d:$header_d" \
            "auth_results.header_s:$header_s" "auth_results.header_a:$header_a" \
            "dkim_signature.raw:$sig_raw" "dkim_signature.d:$sig_d" \
            "dkim_signature.s:$sig_s" "dkim_signature.a:$sig_a" \
            "dkim_signature.b:$sig_b"; do
    key="${pair%%:*}"
    val="${pair#*:}"
    [ -n "$val" ] || missing="$missing $key"
done
[ -z "$missing" ] || fail incomplete-evidence "missing fields:$missing"

if ! printf '%s' "$result" | grep -qi '^pass$'; then
    fail receiver-rejected "auth_results.dkim_result=$result is not pass"
fi
if ! printf '%s' "$raw_ar" | grep -qi 'dkim=pass'; then
    fail verdict-mismatch "auth_results.raw does not contain dkim=pass"
fi
if printf '%s' "$raw_ar" | grep -qiE 'dkim_verify\.py|dkim\.verify\(|python3'; then
    fail invalid-receiver "auth_results.raw names a local verifier instead of a receiving MTA"
fi

# The receiver fields, the DKIM-Signature tags and the configured identities
# must all agree. A mismatch is a real defect, not a skip.
if [ "$header_d" != "$domain" ] || [ "$sig_d" != "$domain" ] || [ "$header_d" != "$sig_d" ]; then
    fail identity-mismatch "d= mismatch: configured=$domain header.d=$header_d dkim.d=$sig_d"
fi
if [ "$header_s" != "$selector" ] || [ "$sig_s" != "$selector" ] || [ "$header_s" != "$sig_s" ]; then
    fail identity-mismatch "s= mismatch: configured=$selector header.s=$header_s dkim.s=$sig_s"
fi
if [ "$header_a" != "$sig_a" ]; then
    fail algorithm-mismatch "header.a=$header_a does not match dkim_signature.a=$sig_a"
fi
case "$sig_a" in
    ed25519-sha256|rsa-sha256) ;;
    *) fail unexpected-algorithm "dkim_signature.a=$sig_a is not ed25519-sha256 or rsa-sha256" ;;
esac
if [ -z "$recipient_address" ] || [ "$recipient_address" != "$to_addr" ]; then
    fail recipient-mismatch "delivery.recipient_address=$recipient_address does not match AG_MAIL_DELIVERY_TO=$to_addr"
fi
if [ -z "$signing_domain" ] || [ "$signing_domain" != "$domain" ]; then
    fail identity-mismatch "delivery.signing_domain=$signing_domain does not match AG_MAIL_DELIVERY_DOMAIN=$domain"
fi
if ! printf '%s' "$commit_sha" | grep -qiE '^[0-9a-f]{7,40}$'; then
    fail invalid-commit "delivery.commit_sha is not a commit hash"
fi

report "evidence.date=$date_utc"
report "evidence.commit=$commit_sha"
report "evidence.receiver=$receiver"
report "evidence.verdict=dkim=pass"
report "evidence.header.d=$header_d s=$header_s a=$header_a b=${header_b:-<omitted-by-receiver>}"
report "evidence.signature.d=$sig_d s=$sig_s a=$sig_a b=<present>"
report "gate.real_mailbox=SATISFIED"
report "RESULT: PASS scope=real-mailbox reason=receiver-authentication-results-dkim-pass"
exit 0

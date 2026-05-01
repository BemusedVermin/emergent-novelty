#!/usr/bin/env bash
# Local imitation of .github/workflows/rust.yml.
# Requires on PATH: cargo-deny, cargo-audit, cargo-hack, rust-code-analysis-cli.

set -uo pipefail

# Color only when stdout is a tty.
if [[ -t 1 ]]; then
    GREEN=$'\e[32m'; RED=$'\e[31m'; YELLOW=$'\e[33m'
    BOLD=$'\e[1m'; DIM=$'\e[2m'; RESET=$'\e[0m'
else
    GREEN=; RED=; YELLOW=; BOLD=; DIM=; RESET=
fi

passed=()
failed=()
skipped=()

step() {
    local name="$1"; shift

    printf '%s>> %s%s\n' "$BOLD" "$name" "$RESET"

    local logfile
    logfile="$(mktemp)"
    local start
    start=$(date +%s)
    "$@" >"$logfile" 2>&1
    local rc=$?
    local elapsed=$(( $(date +%s) - start ))

    if [[ $rc -ne 0 ]] && grep -qE 'command not found|no such command' "$logfile"; then
        printf '   %sSKIP%s  tool not installed (%ds)\n' "$YELLOW" "$RESET" "$elapsed"
        skipped+=("$name")
    elif [[ $rc -eq 0 ]]; then
        printf '   %sPASS%s  %ds\n' "$GREEN" "$RESET" "$elapsed"
        passed+=("$name")
    else
        printf '   %sFAIL%s  %ds\n' "$RED" "$RESET" "$elapsed"
        printf '%s   --- output ---%s\n' "$DIM" "$RESET"
        sed 's/^/     /' "$logfile"
        printf '%s   --------------%s\n' "$DIM" "$RESET"
        failed+=("$name")
    fi
    rm -f "$logfile"
}

printf '%sCI imitation of .github/workflows/rust.yml%s\n\n' "$BOLD" "$RESET"

step "license-check"  cargo deny check --hide-inclusion-graph
step "security-audit" cargo audit --deny warnings
step "format"         cargo fmt --all -- --check
step "clippy"         cargo clippy --all-targets --all-features -- -D warnings
step "msrv"           cargo hack check --rust-version --workspace --all-targets

mkdir -p metrics-output
step "code-analysis"  rust-code-analysis-cli \
    --metrics --output-format json --pr \
    -X 'target/**' \
    -p sim-core sim-main \
    -o metrics-output

step "build" cargo build --verbose
step "test"  cargo test --all-features --all --verbose

# Summary
echo
printf '%sSummary%s  ' "$BOLD" "$RESET"
printf '%s%d passed%s' "$GREEN" "${#passed[@]}" "$RESET"
(( ${#failed[@]}  > 0 )) && printf ', %s%d failed%s'  "$RED"    "${#failed[@]}"  "$RESET"
(( ${#skipped[@]} > 0 )) && printf ', %s%d skipped%s' "$YELLOW" "${#skipped[@]}" "$RESET"
echo

if (( ${#skipped[@]} > 0 )); then
    printf '\n%sSkipped:%s\n' "$YELLOW" "$RESET"
    for s in "${skipped[@]}"; do printf '  - %s\n' "$s"; done
fi

if (( ${#failed[@]} > 0 )); then
    printf '\n%sFailed:%s\n' "$RED" "$RESET"
    for f in "${failed[@]}"; do printf '  - %s\n' "$f"; done
    exit 1
fi
exit 0

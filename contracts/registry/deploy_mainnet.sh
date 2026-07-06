#!/usr/bin/env bash
#
# Deploy + seed the Stellar Registry on MAINNET (Pubnet).
#
# Phases:
#   0. Preflight   — verify network config, the `stellar registry` plugin, and a funded admin key.
#   1. Root        — resolve the root registry; optionally bootstrap it (--bootstrap-root).
#   2. Publish     — publish the registry wasm so sub-registries can deploy `--wasm-name registry`.
#   3. Sub-registries — for each project in mainnet/initial_contracts.json: deploy the named
#                       sub-registry (if missing), then batch-register its contracts.
#   4. Tokens      — (optional, --with-tokens) batch-register mainnet/initial_batch.json
#                    (xlm, cetes) directly into the root registry.
#
# SAFETY: this touches MAINNET and spends real XLM. It runs in --dry-run mode by
# default and only prints the commands it would run. Pass --execute to actually
# submit transactions.
#
set -euo pipefail

# ---------------------------------------------------------------------------
# Config (override via env)
# ---------------------------------------------------------------------------
NETWORK="${STELLAR_NETWORK:-mainnet}"
ADMIN="${REGISTRY_ADMIN:-theahaco}"                 # stellar-cli key alias; must be funded on mainnet
RPC_URL="${STELLAR_RPC_URL:-https://mainnet.sorobanrpc.com}"
PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Public Global Stellar Network ; September 2015}"
BATCH_LIMIT="${BATCH_LIMIT:-10}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Semantic version to publish the registry wasm under (defaults to the crate version).
REGISTRY_VERSION="${REGISTRY_VERSION:-$(awk -F'"' '/^version[[:space:]]*=/ { print $2; exit }' "$SCRIPT_DIR/Cargo.toml")}"
REPO_ROOT="$SCRIPT_DIR/../.."
DATA_DIR="$SCRIPT_DIR/mainnet"
INITIAL_CONTRACTS="$DATA_DIR/initial_contracts.json"
INITIAL_BATCH="$DATA_DIR/initial_batch.json"

# ---------------------------------------------------------------------------
# Args
# ---------------------------------------------------------------------------
DRY_RUN=1
BOOTSTRAP_ROOT=0
WITH_TOKENS=0
DO_PUBLISH=1

usage() {
    cat <<EOF
Usage: $0 [options]

  --execute           Actually submit transactions (default: dry-run, prints only)
  --bootstrap-root    Deploy the root registry if it does not exist yet (see notes)
  --with-tokens       Also batch-register the top-level tokens (initial_batch.json) into root
  --no-publish        Skip the registry-wasm publish step (phase 2)
  -n, --dry-run       Force dry-run (default)
  -h, --help          Show this help

Env overrides: STELLAR_NETWORK ($NETWORK), REGISTRY_ADMIN ($ADMIN),
               STELLAR_RPC_URL, STELLAR_NETWORK_PASSPHRASE, BATCH_LIMIT ($BATCH_LIMIT)

Notes on --bootstrap-root: the root registry's deterministic contract id is derived
from the salt baked into the CLI repo (stellar-registry/cli, crates/stellar-registry-build/.salt).
Set REGISTRY_SALT to that 64-hex value so the deployed root matches the id the
\`stellar registry\` plugin resolves. Without a matching salt, \`fetch-contract-id\`
will not find the root you deploy.
EOF
}

for arg in "$@"; do
    case "$arg" in
        --execute)        DRY_RUN=0 ;;
        -n|--dry-run)     DRY_RUN=1 ;;
        --bootstrap-root) BOOTSTRAP_ROOT=1 ;;
        --with-tokens)    WITH_TOKENS=1 ;;
        --no-publish)     DO_PUBLISH=0 ;;
        -h|--help)        usage; exit 0 ;;
        *) echo "Unknown option: $arg" >&2; usage >&2; exit 1 ;;
    esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m!!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# run: execute (or, in dry-run, print) a state-changing command.
run() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '[dry-run] '
        printf '%q ' "$@"
        printf '\n'
    else
        "$@"
    fi
}

# Common global args passed to every plugin/cli invocation.
net_args=(--network "$NETWORK" --source "$ADMIN")

# ---------------------------------------------------------------------------
# Phase 0 — preflight
# ---------------------------------------------------------------------------
preflight() {
    log "Phase 0: preflight (network=$NETWORK admin=$ADMIN)"

    command -v stellar >/dev/null 2>&1 || die "stellar CLI not found on PATH"
    stellar registry --help >/dev/null 2>&1 \
        || die "the 'stellar registry' plugin is not installed (build/install it from stellar-registry/cli)"
    command -v jq >/dev/null 2>&1 || die "jq not found on PATH"

    # Ensure the target network is configured with the mainnet passphrase.
    if ! stellar network ls 2>/dev/null | grep -qx "$NETWORK"; then
        warn "network '$NETWORK' is not configured; adding it"
        run stellar network add "$NETWORK" \
            --rpc-url "$RPC_URL" \
            --network-passphrase "$PASSPHRASE"
    fi

    # Refuse to silently run against anything that is not the public network.
    if [ "$PASSPHRASE" != "Public Global Stellar Network ; September 2015" ]; then
        warn "passphrase override in effect — this is NOT the standard Pubnet passphrase"
    fi

    stellar keys public-key "$ADMIN" >/dev/null 2>&1 \
        || die "admin key '$ADMIN' not found (add it with 'stellar keys add $ADMIN' or 'stellar keys generate')"

    [ -f "$INITIAL_CONTRACTS" ] || die "missing $INITIAL_CONTRACTS"
    [ -f "$INITIAL_BATCH" ]     || die "missing $INITIAL_BATCH"

    if [ "$DRY_RUN" -eq 1 ]; then
        warn "DRY-RUN: no transactions will be submitted. Re-run with --execute to deploy."
    else
        warn "EXECUTE mode: this WILL submit MAINNET transactions and spend real XLM as '$ADMIN'."
        printf 'Type "mainnet" to continue: '
        read -r confirm
        [ "$confirm" = "mainnet" ] || die "aborted"
    fi
}

# ---------------------------------------------------------------------------
# Phase 1 — root registry
# ---------------------------------------------------------------------------
ROOT_REGISTRY=""

resolve_root() {
    ROOT_REGISTRY="$(stellar registry fetch-contract-id registry "${net_args[@]}" 2>/dev/null || true)"
}

bootstrap_root() {
    log "Phase 1: bootstrapping root registry"
    local wasm="$REPO_ROOT/target/wasm32v1-none/contracts/registry.wasm"
    if [ ! -f "$wasm" ]; then
        warn "registry.wasm not built; building with the contracts profile"
        run stellar contract build --profile contracts --package registry
    fi
    [ -n "${REGISTRY_SALT:-}" ] \
        || die "--bootstrap-root needs REGISTRY_SALT (64-hex) matching the CLI's stellar-registry-build/.salt"

    local manager
    manager="$(stellar keys public-key "$ADMIN")"
    # Root registry: no --root, manager required (constructor enforces ManagerRequired).
    run stellar contract deploy \
        --alias registry \
        --wasm "$wasm" \
        --salt "$REGISTRY_SALT" \
        "${net_args[@]}" \
        -- \
        --admin "$ADMIN" \
        --manager "\"$manager\""
    resolve_root
}

ensure_root() {
    log "Phase 1: resolving root registry"
    resolve_root
    if [ -z "$ROOT_REGISTRY" ]; then
        if [ "$BOOTSTRAP_ROOT" -eq 1 ]; then
            bootstrap_root
        else
            die "root registry not found. Deploy it first, or re-run with --bootstrap-root (needs REGISTRY_SALT)."
        fi
    fi
    [ -n "$ROOT_REGISTRY" ] || [ "$DRY_RUN" -eq 1 ] \
        || die "root registry still unresolved after bootstrap"
    log "Root registry: ${ROOT_REGISTRY:-<unresolved, dry-run>}"
}

# ---------------------------------------------------------------------------
# Phase 2 — publish the registry wasm
# ---------------------------------------------------------------------------
publish_registry_wasm() {
    [ "$DO_PUBLISH" -eq 1 ] || { log "Phase 2: publish skipped (--no-publish)"; return; }
    log "Phase 2: publishing registry wasm"
    local wasm="$REPO_ROOT/target/wasm32v1-none/contracts/registry.wasm"
    if [ ! -f "$wasm" ]; then
        run stellar contract build --profile contracts --package registry
    fi
    # Pass --wasm-name and --binver explicitly. `stellar registry publish` otherwise
    # derives them from the wasm's `name`/`binver` contractmeta, which released
    # registry artifacts do not always carry; supplying both makes publish work for
    # any local wasm. (Each flag alone leaves the other required arg unfilled.)
    run stellar registry publish \
        --wasm "$wasm" \
        --wasm-name registry \
        --binver "$REGISTRY_VERSION" \
        --author "$ADMIN" \
        "${net_args[@]}"
}

# ---------------------------------------------------------------------------
# Phase 3 — per-project sub-registries + batch register
# ---------------------------------------------------------------------------
deploy_subregistry() {
    local name="$1"
    if existing_id="$(stellar registry fetch-contract-id "$name" "${net_args[@]}" 2>/dev/null)" \
        && [ -n "$existing_id" ]; then
        log "sub-registry '$name' already registered ($existing_id); skipping deploy"
    else
        local manager
        manager="$(stellar keys public-key "$ADMIN")"
        run stellar registry deploy \
            --contract-name "$name" \
            --wasm-name registry \
            "${net_args[@]}" \
            -- \
            --admin "$ADMIN" \
            --manager "\"$manager\"" \
            --root "\"$ROOT_REGISTRY\""
    fi
    run stellar registry create-alias "$name" --force "${net_args[@]}"
}

batch_register() {
    local alias="$1" contracts_json="$2"
    run stellar contract invoke --id "$alias" "${net_args[@]}" -- \
        batch-register --contracts "$contracts_json"
    run stellar contract invoke --id "$alias" "${net_args[@]}" -- \
        process_batch --limit "$BATCH_LIMIT"
}

seed_projects() {
    log "Phase 3: sub-registries + batch register"
    while IFS= read -r name; do
        deploy_subregistry "$name"
        local contracts
        contracts="$(jq -c --arg k "$name" '.[] | select(has($k)) | .[$k]' "$INITIAL_CONTRACTS")"
        if [ "$(jq 'length' <<<"$contracts")" -eq 0 ]; then
            log "no contracts for '$name'; skipping batch-register"
            continue
        fi
        batch_register "$name" "$contracts"
    done < <(jq -r '.[] | keys[0]' "$INITIAL_CONTRACTS")
}

# ---------------------------------------------------------------------------
# Phase 4 — top-level tokens into the root registry (optional)
# ---------------------------------------------------------------------------
seed_tokens() {
    [ "$WITH_TOKENS" -eq 1 ] || { log "Phase 4: tokens skipped (pass --with-tokens to enable)"; return; }
    log "Phase 4: registering top-level tokens into root"
    local tokens
    tokens="$(jq -c '.' "$INITIAL_BATCH")"
    if [ "$(jq 'length' <<<"$tokens")" -eq 0 ]; then
        log "no tokens in $INITIAL_BATCH; nothing to do"
        return
    fi
    run stellar registry create-alias registry --force "${net_args[@]}"
    batch_register registry "$tokens"
}

# ---------------------------------------------------------------------------
main() {
    preflight
    ensure_root
    publish_registry_wasm
    seed_projects
    seed_tokens
    log "Done."
}

main

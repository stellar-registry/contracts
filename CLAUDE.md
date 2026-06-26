# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repo holds the **on-chain Stellar Registry contracts** (Soroban smart contracts). It is one of several repos split out of the original `scaffold-stellar` monorepo. The registry manages wasm publication (with semantic versioning) and the deployment of named contract instances.

Related repos:
- `stellar-registry/cli` — the `stellar registry` CLI that interacts with these contracts
- `stellar-registry/ui` — registry frontend
- `stellar-registry/indexer` — registry indexer & API

## Common Commands

```bash
# Install the pinned stellar-cli (v27.0.0) into ./target/bin and set up git hooks
just setup

# Build all contracts with the size-optimized profile
stellar contract build --profile contracts

# Check / lint (library code)
cargo check --workspace
cargo clippy --all-targets

# Run contract tests (build the wasm fixtures first — see Testing)
cargo test --workspace
```

Note: the `justfile` still carries some recipes from the monorepo. Prefer the commands above until it is trimmed to this repo.

## Architecture

### Contracts

| Path | Purpose |
|------|---------|
| `contracts/registry` | The core Registry contract: wasm publication, versioning, named deployments |
| `contracts/test/*` | Test fixtures: `hello_world`, `hello_world_v2`, `hello_world_v3` |

The `feat/registry-tansu-manager` branch additionally contains `contracts/registry-tansu-manager` (a Tansu DAO-gated registry manager), `contracts/hello`, and `contracts/test/tansu-stub`.

## Testing

The registry's tests import compiled fixture wasm via `soroban_sdk::contractimport!` (e.g. `target/stellar/local/hello_world.wasm`). **Build the contracts before running `cargo test`**, otherwise the imports fail to resolve:

```bash
stellar contract build --profile contracts
cargo test --workspace
```

## Build Profile

Contracts use a custom `[profile.contracts]` with aggressive size optimization:
- `opt-level = "z"` (size optimization)
- `lto = true`
- `strip = "symbols"`
- `panic = "abort"`, `codegen-units = 1`

## Cross-repo dependencies

`contracts/registry` has a dev-dependency on the `stellar-registry` crate, consumed from crates.io (published from `stellar-registry/cli`). It is declared as a workspace dependency in the root `Cargo.toml`.

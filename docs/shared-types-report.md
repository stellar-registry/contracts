# Report: Is it worth pulling more types out of the registry into a shared dep?

**Context.** We extracted `NormalizedName` from the `registry` contract crate into a new
`no_std` rlib, `stellar-registry-types`, so other Soroban contracts can reuse it. The
first consumer is Tansu, which now validates project names with
`stellar_registry_types::NormalizedName` instead of the SorobanDomain contract. This
report evaluates whether *more* registry types should follow.

## TL;DR

Extract on **proven cross-contract demand**, not on principle. `NormalizedName` cleared the
bar because a second contract (Tansu) needed exactly it. Of the remaining registry types,
only a small **Tier 1** has plausible reuse value today; most are storage/implementation
details that should stay in the contract crate. Recommendation: keep
`stellar-registry-types` deliberately small, add types only when a concrete second consumer
appears, and treat the published-crate boundary as an API commitment (semver, no_std,
SDK-version-flexible).

## What's already shared

| Crate | Contents | Notes |
|---|---|---|
| `stellar-registry-types` (this repo, new) | `NormalizedName`, `NameError` | `no_std` rlib, `soroban-sdk = ">=25.3.1, <27"` so it builds for consumers on SDK 25 *or* 26 (Tansu is on 26). Uses only version-stable APIs + `contracterror`. |
| `stellar-registry` (crates.io 0.0.10) | re-exports `import_contract_client!` | Macro-only; exports no domain types. A separate crate/repo (scaffold-stellar). |

## Candidate inventory

| Type / item | Where | `#[contracttype]`? | Logic vs data | Cross-contract reuse value | Verdict |
|---|---|---|---|---|---|
| `NormalizedName` | `registry/src/name.rs` | no (newtype + `IntoVal`/`TryFromVal`) | validation/normalization | **High** — proven (Tansu) | **Done** |
| `Error` (registry) | `registry/src/error.rs` | `scerr` enum | data | Medium — callers decoding XCC failures | Tier 1 (selective) |
| Semver/version validation | `registry` (uses `semver` crate) | n/a | logic | Medium — any registry-like versioned store | Tier 1 (if a consumer needs it) |
| `PublishedWasm` | `registry/src/registry/wasm.rs` | yes | data + getters | Low/Medium — only if others query wasm metadata | Tier 2 |
| Event structs (`Publish`, `Deploy`, …) | `registry/src/events.rs` | `contractevent` | data | Low — indexers read these off-chain, not contracts | Tier 2 |
| `ContractEntry`, storage keys (`WasmKey`, `ContractKey`, `HashKey`, `Manager`) | `registry/src/storage.rs` | mixed | storage-coupled | None — ledger-model internals | Keep internal |
| Traits (`Deployable`, `Batchable`, `Manageable`, `Redeployable`, `Publishable`, `Proxyable`, `ToStorageKey`) | `registry/src/...` | n/a | contract behavior | None for consumers | Keep internal |

## Recommendations

### Tier 1 — extract only when a second consumer is concrete

- **`Error` (selective).** Don't move the whole enum — it's registry-specific and large
  (`result_large_err` is even allow-listed). If a consumer needs to *decode* a specific
  registry failure from a cross-contract call, extract just that variant (as we did with
  `NameError`) and map it. Moving the full enum would couple every consumer to registry
  internals and churn on every new variant.
- **Version/semver validation.** If another contract wants the same "new version must be
  greater than current" / cargo-semver parsing, lift that helper (not the storage) into
  `stellar-registry-types`. No consumer needs it yet, so hold.

### Tier 2 — defer

- **`PublishedWasm`, event structs.** Reuse is hypothetical. Event structs are consumed by
  off-chain indexers (TypeScript bindings already cover that); a Rust contract rarely needs
  the struct. Extract only if an on-chain consumer materializes.

### Keep internal — do not extract

- Storage keys, `ContractEntry`, the `Storage` struct, and the capability traits are bound
  to the registry's ledger layout. Sharing them leaks internals and creates a versioning
  trap (a storage tweak becomes a breaking change for every dependent).

## Why "demand-driven" beats "extract everything"

1. **SDK-version skew is the real tax.** Registry is on `soroban-sdk` 25.x; Tansu on 26.x. A
   shared crate must compile against *both*, which forces a wide version range and a
   conservative, version-stable API surface (we dropped `soroban-sdk-tools` from
   `stellar-registry-types` for this reason and used built-in `contracterror`). Every type
   added widens that compatibility burden.
2. **The contract crate is `publish = false` + `cdylib`.** Anything kept there is free to
   change. Anything promoted to the shared rlib becomes a semver-committed public API.
3. **Storage types are liabilities, not assets, when shared.** Their value is precisely that
   they can change with the ledger layout; sharing removes that freedom.

## Concrete next steps

1. Keep `stellar-registry-types` at just `NormalizedName` + `NameError` for now.
2. When the registry-tansu-manager (or another contract) needs to decode a specific registry
   error or reuse version validation, extract *that one thing* using the `NameError` pattern
   (small dedicated type + `From` mapping in the contract), not a bulk move.
3. Before the first real external (non-monorepo) consumer, give `stellar-registry-types` a
   crates.io release and a semver policy; until then a pinned git rev (what Tansu uses) is
   fine.

# stellar-registry contracts

[![Apache 2.0 licensed](https://img.shields.io/badge/license-apache%202.0-blue.svg)](LICENSE)

On-chain smart contracts for the **Stellar Registry** — the infrastructure layer
between "I wrote a smart contract" and "the ecosystem can safely use my smart
contract." Built for [Soroban](https://soroban.stellar.org) on the
[Stellar](https://stellar.org) blockchain.

## Related repositories

- **On-chain contracts** (this repo): [stellar-registry/contracts](https://github.com/stellar-registry/contracts)
- **CLI**: [stellar-registry/cli](https://github.com/stellar-registry/cli)
- **Frontend**: [stellar-registry/ui](https://github.com/stellar-registry/ui)
- **Indexer & API**: [stellar-registry/indexer](https://github.com/stellar-registry/indexer)

## Contracts

| Path | Description |
|------|-------------|
| [`contracts/registry`](./contracts/registry) | The core Registry contract: publishes Wasm with versioning and deploys named instances |
| `contracts/test/*` | Test fixtures (`hello_world`, `hello_world_v2`, `hello_world_v3`) used by the registry's test suite |

The `feat/registry-tansu-manager` branch additionally contains
`contracts/registry-tansu-manager` (a Tansu DAO-gated registry manager),
`contracts/hello`, and `contracts/test/tansu-stub`.

## What the Registry does

- Register contract names for publishing
- Publish contract binaries with version management
- Fetch contract binaries and metadata
- Deploy published contracts to the blockchain
- Retrieve deployment statistics for contracts
- Manage contract ownership and redeployment

It separates **Wasm publication** (reusable code), **contract deployment**
(named instances), and **local installation** (CLI aliases handled by the
[CLI](https://github.com/stellar-registry/cli)).

## Building

Contracts build with the size-optimized `contracts` profile:

```bash
stellar contract build --profile contracts
```

Tests rely on prebuilt fixture Wasm artifacts (imported via
`soroban_sdk::contractimport!`), so build the workspace before running
`cargo test`.

## Documentation

- [Registry Guide](https://scaffoldstellar.com/docs/registry)
- [CLI Commands](https://scaffoldstellar.com/docs/cli)

## License

Licensed under the Apache-2.0 License — see [LICENSE](./LICENSE) for details.

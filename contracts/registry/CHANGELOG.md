# Changelog

All notable changes to the `registry` contract are documented here. Versions
are the wasm versions published to the Stellar Registry; the format follows
[Keep a Changelog](https://keepachangelog.com).

## [0.6.6] - 2026-10-01

### 🐛 Bug Fixes

- Test PR release and publish (#56)

## [0.6.5] - 2026-10-01

### 🚀 Features

- *(registry)* Mainnet deploy script + verified mainnet seed data (#14)
- Add known assets data (#21)
- Call `preauthorize_author_transfer` to transfer wasm authorship (#34)
- Register named G-addresses (accounts) (#37)
- Manage named G-address (account) entries (#38)

### 🐛 Bug Fixes

- Path to .salt file in verified_contract_id (#490)
- *(deploy_mainnet)* Rm trailing comma because json (#15)
- *(deploy_mainnet)* Don't build wasm (#18)
- *(deploy_mainnet)* Quotation typo (#19)
- MAX_BUMP for ttl set to actual max instead of min (#35)
- *(registry)* Fix max_ttl and bump version (#36)
- *(registry)* Clamp preauth transfer TTL to network max_ttl (#50)

### 📚 Documentation

- Rewrite README and update repo references for standalone split

## [0.6.2] - 2026-04-27

### 🚀 Features

- *(registry)* Update to new contract id for registry (#478)

## [0.6.1] - 2026-04-24

### 🐛 Bug Fixes

- *(registry)* Root sub_reg events in constructor should go first (#485)

## [0.6.0] - 2026-04-22

### 🚀 Features

- Deploy from external subregistries (#484)

## [0.5.1] - 2026-04-17

### 🐛 Bug Fixes

- Rename the name registered contract to root (#480)

## [0.5.0] - 2026-04-16

### 🚀 Features

- Mark contracts (#461)
- *(registry)* Publish SubRegistry event on sub-registry deploy (#470)

### 🚜 Refactor

- *(registry)* Use soroban-sdk-tools::scerr for error enum (#471)

### 📚 Documentation

- *(contracts/registry)* Link other repos (#464)

## [0.4.1] - 2026-03-19

### 🚀 Features

- *(registry)* Validate names used in registry; to prevent potential issues when used with rust imports (#69)
- [**breaking**] Use semver::Version instead of custom type (#71)
- Allow dashes and ensure that first char in name is ascii alphabetic (#73)
- Update resolution of contract id to prepare for deploying to main net (#74)
- *(registry)* Update readme with instructions for mainnet; Update to newest loam_sdk; Use correct contract Ids (#86)
- Add Cargo metadata (#108)
- *(registry)* Ensure that all names are converted to kebab case (#175)
- *(registry-cli)* Add download, create-alias, upgrade commands (#176)
- Add network in target wasm path (#213)
- [**breaking**] Refactor registry contract (#204)
- Add managed contracts and `stellar-registry-build` (#322)
- *(registry)* Add batch registration and contract management (#404)
- *(registry)* Add hash and SAC validation error variants (#415)

### 🐛 Bug Fixes

- *(registry)* `upgrade_contract` now requires admin at the root level (#177)
- Registry deploy with no constructor should use empty tuple not void (#179)
- Update all links (#198)
- *(contracts/registry)* Clean up based on Audit feedback (#279)
- Use pedantic clippy and apply suggestions (#379)
- Update code for stellar-cli v25, soroban-sdk v25, and admin-sep API changes (#383)

## [0.0.1-alpha] - 2025-05-20

### 🚀 Features

- First real initial commit (#2)
- [**breaking**] Remove metadata and require version when publishing (#17)
- Initial deploy work (#57)
- [**breaking**] Minimize storage (#53)
- Add stellar-registry crate (#64)
- Protect the name `registry` and ensure that a published Wasm's version is greater than the current (#65)

### 🐛 Bug Fixes

- Remove claimable contract logic; rename wasm_name contract_name (#14)



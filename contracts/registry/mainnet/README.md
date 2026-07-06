# Mainnet registry seed data

Contract IDs for the initial mainnet (Pubnet) registry deployment. Every address
below was resolved from an authoritative source **and verified live on Pubnet**
(stellar.expert public API returned HTTP 200 for each contract id).

- `owner` for every row is `GAMPJROHOAW662FINQ4XQOY2ULX5IEGYXCI4SMZYE75EHQBR6PSTJG3M` (theahaco).
- Network passphrase: `Public Global Stellar Network ; September 2015`.
- `deploy_mainnet.sh` (one dir up) consumes these two files.

## `initial_contracts.json` — per-project sub-registries

| Project | Name | Contract ID | Source | Confidence |
|---------|------|-------------|--------|------------|
| circle | usdc | `CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75` | stellar.expert asset API (Circle USDC SAC) | high |
| soroswap | router | `CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH` | soroswap/core `public/mainnet.contracts.json` | high |
| soroswap | factory | `CA4HEQTL2WPEUYKYKCDOHCDNIV4QHNJ7EL4J4NQ6VADP7SYHVRYZ7AW2` | soroswap/core `public/mainnet.contracts.json` | high |
| blend | pool-factory-v2 | `CDSYOAVXFY7SM5S64IZPPPYB4GVGGLMQVFREPSQQEZVIWXX5R23G4QSU` | blend-capital/blend-utils `mainnet.contracts.json` (`poolFactoryV2`) | high |
| blend | backstop-v2 | `CAQQR5SWBXKIGZKPBZDH3KM5GQ5GUTPKB7JAFCINLZBC5WXPJKRG3IM7` | blend-utils `mainnet.contracts.json` (`backstopV2`) | high |
| blend | fixed-v2-pool | `CAJJZSGMMM3PD7N33TAPHGBUGTB43OC73HVIK2L2G6BNGGGYOSSYBXBD` | blend-utils `mainnet.contracts.json` (`FixedV2`) | high |
| blend | cetes-pool | `CDMAVJPFXPADND3YRL4BSM3AKZWCTFMX27GLLXCML3PD62HEQS5FPVAI` | on-chain (Blend V2 wasm + dominant CETES reserve) | medium |
| blend | usdc | `CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75` | blend-utils `mainnet.contracts.json` (`USDC` = Circle USDC SAC) | high |
| defindex | factory | `CDKFHFJIET3A73A2YN4KV7NSV32S6YGQMUFH3DNJXLBWL4SKEGVRNFKI` | paltalabs/defindex `public/mainnet.contracts.json` (`defindex_factory`) | high |
| defindex | usdc-vault | `CBNKCU3HGFKHFOF7JTGXQCNKE3G3DXS5RDBQUKQMIIECYKXPIOUGB2S3` | on-chain vault #7 (Beans USDC), factory-enumerated | medium |
| defindex | xlm-vault | `CCB2AR5X3KP4WQKE7HNSUSDS7SHFMC2WPVSZ2ZXJ6DHXOKHFFKOZE6GK` | on-chain vault #95 (Peridot XLM), factory-enumerated | medium |
| defindex | cetes-vault | `CANBU7T77SCJOOAU6VQAOGR7DN36JBQFUN56XS2WA2VPJYUSRUBIPYDS` | on-chain vault #96 (Neko Cetes), factory-enumerated | medium |
| defindex | cetes-blend-strategy | `CAZ3LLLKPWEOVK6K4G5NCQ2VXWABLFIPKKNMN5GLKMZKEN7JSKTEMIKN` | paltalabs/defindex `mainnet.contracts.json` (`cetes_blend_autocompound_etherfuse_strategy`) | high |

`oz` is intentionally empty (matches the testnet seed).

## `initial_batch.json` — top-level tokens (registered into the root registry)

| Name | Contract ID | Source | Confidence |
|------|-------------|--------|------------|
| xlm | `CAS3J7GYLGXMF6TDJBBYYSE3HQ6BBSMLNUQ34T6TZMYMW2EVH34XOWMA` | native XLM SAC (deterministic) | high |
| cetes | `CAL6ER2TI6CTRAY6BFXWNWA7WTYXUXTQCHUBCIBU5O6KM3HJFG6Z6VXV` | Etherfuse CETES SAC (issuer `GCRYUGD5NVARGXT56XEZI5CIFCQETYHAPQQTHO2O3IQZTHDH4LATMYWC`, etherfuse.com) | high |

## Decisions taken vs. the testnet seed (`../initial_contracts.json`)

1. **`blend/testnet-v2-pool` → `blend/fixed-v2-pool`.** The testnet slot name is a
   leftover. On mainnet it maps to Blend's flagship **FixedV2** pool (USDC/XLM/BLND/wETH/wBTC,
   the largest Blend pool on Pubnet). Renamed for accuracy — no consumers exist yet.
2. **DeFindex vaults are third-party/partner vaults** (Beans / Peridot / Neko), selected by
   on-chain TVL + activity. The DeFindex-team's own USDC/XLM vaults exist but currently hold 0.
   No per-asset vault registry is published upstream, so these are a curated choice.
3. **`blend/cetes-pool` (medium).** Not in any upstream deployment file; identified on-chain
   (runs the Blend V2 lending-pool wasm and holds by far the largest CETES reserve of any pool).

## Re-verifying an address

```bash
curl -s "https://api.stellar.expert/explorer/public/contract/<CONTRACT_ID>" | jq '{created, wasm, asset}'
```

A `created` timestamp = the contract exists on Pubnet. `asset` present + no `wasm` = a Stellar
Asset Contract (SAC); a `wasm` hash = a deployed wasm contract.

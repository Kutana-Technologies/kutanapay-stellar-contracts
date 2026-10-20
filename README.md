# KutanaPay Stellar Contracts

[![test](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/actions/workflows/test.yml/badge.svg)](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/actions/workflows/test.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Soroban smart contracts used by **KutanaPay Trade Secure** on the Stellar network. Trade Secure is a trade-assurance product for business-to-business deals. These contracts publish two kinds of verifiable record on-chain:

| Contract | Purpose | Stores | Never stores |
|---|---|---|---|
| [KYB Attestation Registry](kyb-attestation-registry/) | Public know-your-business (KYB) standing for a counterparty's Stellar account | Tier, expiry timestamp, revocation flag | Names, documents, contact details, or any other personal data |
| [Insurance Record](insurance-record/) | Tamper-evident anchors for trade-insurance documents | SHA-256 digests of policy, claim-decision and payout documents, plus a status code | The documents themselves, claim narratives, or bank details |

Both contracts are **registries, not vaults**. They hold no tokens and cannot move funds. Deal escrow is handled separately by [Trustless Work](https://www.trustlesswork.com/) escrow contracts, which are not part of this repository.

## Status

| | |
|---|---|
| Version | `0.2.0` (see [CHANGELOG](CHANGELOG.md)) |
| Network | Stellar **testnet** only ([deployments](docs/TESTNET.md)) |
| Audit | Not yet audited. An independent review is planned before any mainnet deployment. |
| Soroban SDK | `22.0.11` |

> [!IMPORTANT]
> Version 0.2.0 has a single upgrade administrator and known limitations, which are documented in the [security model](docs/SECURITY.md). Do not rely on these contracts on mainnet until an audited release is published.

## Quick start

### Prerequisites

| Tool | Version | Notes |
|---|---|---|
| [rustup](https://rustup.rs/) | any | Installs the pinned toolchain automatically from [`rust-toolchain.toml`](rust-toolchain.toml) (Rust `1.98.1`, target `wasm32v1-none`) |
| [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) | `28.0.0` | Required for reproducible release builds and for deployment |

### Build

```bash
stellar contract build --locked
```

This produces the following files in `target/wasm32v1-none/release/`:

| File | SHA-256 (v0.2.0) |
|---|---|
| `kyb_attestation_registry.wasm` | `6c06e0ecafe9616903f0ceef2a8ada208aafb26404ace1deb39ee89720218747` |
| `insurance_record.wasm` | `ffeae58c889f2835cc9716e81724c229703584c26446f18862979a9669563452` |

Builds are reproducible: the pinned toolchain and Stellar CLI `28.0.0` produce byte-identical WASM, matching the code deployed on testnet. See [Verify on-chain state](docs/VERIFY.md#1-confirm-the-deployed-code-matches-this-repository).

### Test

```bash
cargo test --workspace --locked
```

See the [test plan](docs/TESTING.md) for what is covered.

### Deploy to testnet

The address passed as `--admin` becomes the **only** account that can write records or upgrade the contract. Choose it carefully (see [Admin key management](docs/SECURITY.md#admin-key-management)).

```bash
stellar keys generate deployer --network testnet --fund
ADMIN=<admin G-address>

stellar contract deploy \
  --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
  --network testnet --source-account deployer \
  -- --admin "$ADMIN"

stellar contract deploy \
  --wasm target/wasm32v1-none/release/insurance_record.wasm \
  --network testnet --source-account deployer \
  -- --admin "$ADMIN"
```

### Upgrade in place

`upgrade` replaces the contract's code while keeping its contract ID and storage. It must be authorized by the admin.

```bash
HASH=$(stellar contract upload \
  --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
  --network testnet --source-account admin)

stellar contract invoke --id "$KYB_CONTRACT_ID" \
  --network testnet --source-account admin \
  -- upgrade --new_wasm_hash "$HASH"
```

The same steps apply to `insurance_record.wasm`.

## Documentation

| Document | Contents |
|---|---|
| [Contract interfaces](docs/INTERFACES.md) | Functions, data types, storage layout, and embedded metadata |
| [Integration guide](docs/INTEGRATION.md) | How Trade Secure deal events map to contract calls, and how identifiers are hashed |
| [Security model](docs/SECURITY.md) | Trust assumptions, authorization, upgrade policy, known limitations |
| [Verify on-chain state](docs/VERIFY.md) | Independently check deployed code, KYB standing, and insurance records |
| [Testnet deployments](docs/TESTNET.md) | Current contract IDs and public role addresses |
| [Test plan](docs/TESTING.md) | Test coverage and how to run it |

## Repository layout

```
.
├── kyb-attestation-registry/   KYB Attestation Registry contract
├── insurance-record/           Insurance Record contract
├── docs/                       Technical documentation
├── Cargo.toml                  Workspace manifest and release profile
└── rust-toolchain.toml         Pinned Rust toolchain
```

## Security

Please do not report vulnerabilities in public issues. See [SECURITY.md](SECURITY.md) for how to report them privately.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 Kutana Technologies. See [NOTICE](NOTICE).

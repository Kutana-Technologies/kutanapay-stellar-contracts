# KutanaPay Stellar Contracts

[![test](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/actions/workflows/test.yml/badge.svg)](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/actions/workflows/test.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Two [Soroban](https://developers.stellar.org/docs/build/smart-contracts/overview) smart contracts for publishing verifiable records on the Stellar network.

| Contract | What it records |
|---|---|
| [KYB Attestation Registry](kyb-attestation-registry/) | Whether a Stellar account has passed a know-your-business (KYB) review, at which tier, and until when |
| [Insurance Record](insurance-record/) | SHA-256 commitments to an insurance policy, a claim decision and a payout receipt, linked by 32-byte identifiers |

Both contracts are **registries**. They hold no tokens, cannot move funds, and store no personal data: only account addresses, short codes, timestamps and 32-byte digests.

## Features

- **Single administrator** with two-step, cancellable transfer of the role
- **Write-integrity rules:** a revocation persists until it is explicitly reinstated, a payout requires an approved claim, and recorded receipts are immutable
- **Idempotent writes:** replaying an identical write succeeds, so callers can retry safely
- **An event for every state change**, for indexing and audit
- **Self-maintaining storage lifetime**, plus extension functions that anyone can call
- **Typed errors** with stable numeric codes
- **Reproducible builds:** the WASM can be rebuilt byte for byte from source

## Quick start

### Prerequisites

| Tool | Version | Notes |
|---|---|---|
| [rustup](https://rustup.rs/) | any | Installs the toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml) (Rust `1.98.1`, target `wasm32v1-none`) |
| [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) | `28.0.0` | Builds release WASM and deploys contracts |

### Build

```bash
stellar contract build --locked
```

The WASM files are written to `target/wasm32v1-none/release/`:

| File | SHA-256 (v0.3.0) |
|---|---|
| `kyb_attestation_registry.wasm` | `2884f8c01afabba124acd1a7800dbb561c4a69dd137eadb85d9fa31acc0d4243` |
| `insurance_record.wasm` | `ee4d009e61dfffc4dc387d2e0c4086450026c01b6d0192f46553e01219e5806c` |

### Test

```bash
stellar contract build --locked   # the upgrade tests load the release WASM
cargo test --workspace --locked
```

### Deploy

```bash
stellar keys generate deployer --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
  --network testnet --source-account deployer \
  -- --admin <ADMIN_ADDRESS>
```

The `--admin` address becomes the only account that can write records or upgrade the contract. See the [deployment guide](docs/DEPLOYMENT.md) before deploying to any network you rely on.

## Documentation

| Guide | Read it to… |
|---|---|
| [Overview](docs/OVERVIEW.md) | Understand the roles, records and lifecycles |
| [Contract reference](docs/INTERFACES.md) | Look up functions, types, errors, events and storage |
| [Integration guide](docs/INTEGRATION.md) | Write to or read from the contracts from an application |
| [Deployment and operations](docs/DEPLOYMENT.md) | Deploy, upgrade, rotate the admin, and maintain storage |
| [Security model](docs/SECURITY.md) | Understand the trust assumptions and the guarantees enforced |
| [Verification guide](docs/VERIFY.md) | Independently check deployed code and on-chain records |
| [Testing](docs/TESTING.md) | Run the tests and see what they cover |
| [Testnet deployments](docs/TESTNET.md) | Find the reference contract IDs |

## Repository layout

```
.
├── kyb-attestation-registry/   KYB Attestation Registry contract
├── insurance-record/           Insurance Record contract
├── upgrade-tests/              In-place upgrade tests from the previous release
├── docs/                       Documentation
├── Cargo.toml                  Workspace manifest and release profile
└── rust-toolchain.toml         Pinned Rust toolchain
```

## Status

| | |
|---|---|
| Version | `0.3.0` ([changelog](CHANGELOG.md)) |
| Networks | Testnet ([deployments](docs/TESTNET.md)) |
| Audit | An independent security review is planned before mainnet deployment |

## Contributing and security

- **Contributing:** see [CONTRIBUTING.md](CONTRIBUTING.md).
- **Vulnerabilities:** report them privately, as described in [SECURITY.md](SECURITY.md). Do not open public issues for them.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 Kutana Technologies. See [NOTICE](NOTICE).

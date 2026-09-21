# KutanaPay Stellar contracts

Apache-2.0 Soroban contracts for **Trade Secure on Stellar** (SCF Build Award, Integration Track).

This repository is the open-source home of the two contracts KutanaPay authors. It is written so a developer outside KutanaPay can **build, test, and deploy both contracts from this repo alone**.

Escrow funds are **not** held here. USDC escrow uses [Trustless Work](https://www.trustlesswork.com/) audited contracts. These crates only store KYB standing and insurance document hashes.

| Crate | What it stores | Never stores |
|-------|----------------|--------------|
| [`kyb-attestation-registry`](kyb-attestation-registry/) | Subject G-address → tier, expiry, revoked | Names, documents, emails, KYC PII |
| [`insurance-record`](insurance-record/) | Deal/claim ids (hashed) → policy/claim/payout hashes + status | Policy PDFs, claim narratives, payout bank details |

**License:** [Apache License 2.0](LICENSE).

## Documentation (SCF)

| Doc | Covers |
|-----|--------|
| [docs/INTERFACES.md](docs/INTERFACES.md) | SEP-46 metadata and SEP-48 function specs |
| [docs/INTEGRATION.md](docs/INTEGRATION.md) | Architecture: deal events → Trustless Work escrow + these registries |
| [docs/VERIFY.md](docs/VERIFY.md) | Independently verify attestations, escrow, KYB, and insurance |
| [docs/TESTNET.md](docs/TESTNET.md) | Current testnet contract ids |
| [docs/TESTING.md](docs/TESTING.md) | Contract test plan (unauthorized write, revoke) |
| [docs/SECURITY.md](docs/SECURITY.md) | Auth, upgrade, what a defect can and cannot do |

## Prerequisites

- [Rust](https://rustup.rs/) (see `rust-toolchain.toml`)
- target `wasm32v1-none`: `rustup target add wasm32v1-none`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli) (`stellar`)

## Build

```bash
stellar contract build
```

WASM lands in `target/wasm32v1-none/release/`:

- `kyb_attestation_registry.wasm`
- `insurance_record.wasm`

```bash
cargo test --workspace
```

## Deploy (testnet)

Create a funded identity once (`stellar keys generate deploy-admin --network testnet` and friendbot). `admin` is frozen in `__constructor` and is the only address that may write or upgrade.

```bash
ADMIN=$(stellar keys address deploy-admin)

stellar contract deploy \
  --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
  --network testnet \
  --source-account deploy-admin \
  -- \
  --admin "$ADMIN"

stellar contract deploy \
  --wasm target/wasm32v1-none/release/insurance_record.wasm \
  --network testnet \
  --source-account deploy-admin \
  -- \
  --admin "$ADMIN"
```

Print the SEP-48 spec of a live id:

```bash
stellar contract info interface --network testnet --id C...
```

## Upgrade (same `C…` id)

```bash
stellar contract upload \
  --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
  --network testnet \
  --source-account deploy-admin

stellar contract invoke \
  --id "$KYB_CONTRACT_ID" \
  --network testnet \
  --source-account deploy-admin \
  -- upgrade --new_wasm_hash <HASH>
```

Same pattern for `insurance_record.wasm`.

# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions correspond to the `binver` embedded in each contract.

## [0.3.0] - Unreleased

The 0.2.0 function signatures and storage layout are kept, so deployed contracts can be upgraded in place. See [Upgrading from 0.2.0](docs/DEPLOYMENT.md#upgrading-from-020).

| WASM | SHA-256 |
|---|---|
| `kyb_attestation_registry.wasm` | `2884f8c01afabba124acd1a7800dbb561c4a69dd137eadb85d9fa31acc0d4243` |
| `insurance_record.wasm` | `ee4d009e61dfffc4dc387d2e0c4086450026c01b6d0192f46553e01219e5806c` |

### Added

- Two-step, cancellable admin transfer: `propose_admin`, `accept_admin`, `cancel_admin_transfer` and `get_pending_admin`.
- Events for every state change, admin change and upgrade.
- KYB: `is_valid` and `reinstate_attestation`.
- Storage lifetime is extended on every write. New functions `extend_ttl` and per-record extension can be called by anyone.
- Typed contract errors with stable codes.
- Upgrade tests from the 0.2.0 release WASM.

### Changed

- KYB: `write_attestation` requires `expires_at` to be in the future, and rejects revoked subjects. Use `reinstate_attestation` to clear a revocation.
- KYB: `revoke_attestation` records unknown subjects as revoked.
- Insurance: `record_payout_receipt` requires an approved decision for the same deal. Receipts and paid decisions are final.
- Insurance: decisions must be `approved` or `denied`, document hashes must be non-zero, and a claim keeps its original `deal_id`.

## [0.2.1] - Unreleased

Build and test infrastructure only. The release WASM is byte-identical to 0.2.0.

### Added

- CI runs the unit tests, a format check, lints and a reproducible release build.
- Authorization regression tests.

### Changed

- The pinned Rust toolchain is now 1.98.1, so release builds reproduce the 0.2.0 WASM.

## [0.2.0] - 2026-09-21

Commit [`768f48a`](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/tree/768f48a), built with Rust 1.98.1 and Stellar CLI 28.0.0.

| WASM | SHA-256 |
|---|---|
| `kyb_attestation_registry.wasm` | `6c06e0ecafe9616903f0ceef2a8ada208aafb26404ace1deb39ee89720218747` |
| `insurance_record.wasm` | `ffeae58c889f2835cc9716e81724c229703584c26446f18862979a9669563452` |

### Added

- KYB Attestation Registry and Insurance Record contracts, with admin-only writes, public reads and in-place upgrade.
- SEP-46 metadata.

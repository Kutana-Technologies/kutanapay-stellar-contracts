# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions refer to the `binver` value embedded in each contract.

## [Unreleased]

Contract code is unchanged. The release WASM is byte-identical to 0.2.0.

### Fixed

- The unit tests are now compiled and run. Previously `test.rs` was not declared as a module, and the `testutils` feature was not enabled for tests.
- `cargo test` compiles again. `ed25519-dalek` is locked to 2.x; 3.x pulls in an incompatible `rand_core` used by `soroban-env-host` 22.
- The pinned toolchain is now `1.98.1`, the version that produced the deployed WASM. Release builds are now reproducible from the repository.

### Added

- Authorization regression tests covering non-admin revoke, claim and payout writes, a correct admin address without the admin's signature, and non-admin `upgrade`.
- Continuous integration now runs `rustfmt`, `clippy -D warnings`, and `cargo test`, then builds the WASM with a checksum-verified Stellar CLI and publishes the WASM hashes. Actions are pinned to commit SHAs.
- `SECURITY.md` vulnerability reporting policy, this changelog, and per-contract READMEs.

### Changed

- Documentation rewritten. Verification commands were corrected (`--source-account` is required for `invoke`), and the identifier hashing rules and known limitations are now documented.

## [0.2.0] - 2026-09-21

### Added

- `kyb-attestation-registry`: admin-only `write_attestation`, `revoke_attestation`, public `get_attestation`, and in-place `upgrade`.
- `insurance-record`: admin-only `record_policy`, `record_claim_decision`, `record_payout_receipt`, public `get_policy` and `get_claim`, and in-place `upgrade`.
- SEP-46 contract metadata (`name`, `org`, `binver`, `desc`).

| WASM | SHA-256 |
|---|---|
| `kyb_attestation_registry.wasm` | `6c06e0ecafe9616903f0ceef2a8ada208aafb26404ace1deb39ee89720218747` |
| `insurance_record.wasm` | `ffeae58c889f2835cc9716e81724c229703584c26446f18862979a9669563452` |

[Unreleased]: https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/compare/768f48a...HEAD
[0.2.0]: https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/tree/768f48a

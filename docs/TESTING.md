# Test Plan

## Running the tests

```bash
cargo test --workspace --locked
```

The tests use the Soroban SDK test environment (`soroban-sdk` with the `testutils` feature), which runs each contract inside a local Soroban host. No network access is needed.

Continuous integration runs, on every push to `main` and every pull request:

| Step | Command |
|---|---|
| Formatting | `cargo fmt --all -- --check` |
| Lints | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Unit tests | `cargo test --workspace --locked` |
| Release build | `stellar contract build --locked` (Stellar CLI 28.0.0), with the WASM SHA-256 hashes published in the job summary |

## Coverage

### KYB Attestation Registry ([`src/test.rs`](../kyb-attestation-registry/src/test.rs))

| Test | Verifies |
|---|---|
| `constructor_sets_admin` | The admin is stored at deployment |
| `admin_can_write_and_read_standing` | Write then read returns tier, expiry and `revoked = false` |
| `admin_can_revoke` | Revocation sets `revoked = true` |
| `rewrite_clears_revoked_flag` | A write after revocation replaces the row |
| `get_missing_subject_is_none` | An unknown subject returns `None` |
| `write_requires_admin_auth` | A write succeeds with exactly the admin's authorization for that call |
| `non_admin_cannot_write` | A write from another address fails with `not admin` |
| `non_admin_cannot_revoke` | A revoke from another address fails with `not admin` |
| `admin_address_without_admin_signature_is_rejected` | Passing the admin address without its signature fails with `Error(Auth, InvalidAction)` |
| `stranger_cannot_upgrade` | `upgrade` signed by another address fails with `Error(Auth, InvalidAction)` |

### Insurance Record ([`src/test.rs`](../insurance-record/src/test.rs))

| Test | Verifies |
|---|---|
| `constructor_sets_admin` | The admin is stored at deployment |
| `admin_can_record_and_read_policy` | A policy write and read round-trip |
| `claim_decision_then_payout_receipt` | The payout receipt keeps the recorded decision and hash |
| `missing_policy_is_none` | An unknown deal returns `None` |
| `non_admin_cannot_record_policy` | Fails with `not admin` |
| `non_admin_cannot_record_claim_decision` | Fails with `not admin` |
| `non_admin_cannot_record_payout_receipt` | Fails with `not admin` |
| `admin_address_without_admin_signature_is_rejected` | Fails with `Error(Auth, InvalidAction)` |
| `stranger_cannot_upgrade` | Fails with `Error(Auth, InvalidAction)` |

## Not covered by unit tests

- **Consumer-side validity checks**, such as KYB expiry. These belong to consuming applications.
- **Network behavior** such as TTL expiry and fees. Test it on testnet.
- **Upgrade to a real new WASM.** Exercise it on testnet before each release.

## Dependency note

`soroban-env-host` 22 accepts any `ed25519-dalek` from 2.0 onward. Version 3.x depends on a newer `rand_core` and breaks the test build. `Cargo.lock` therefore keeps `ed25519-dalek` at 2.x. Avoid running `cargo update` without `--locked` checks, or re-pin it with:

```bash
cargo update -p ed25519-dalek --precise 2.2.0
```

# Testing

## Run the tests

```bash
stellar contract build --locked     # required first: the upgrade tests load the release WASM
cargo test --workspace --locked
```

The tests run each contract in a local Soroban host, using `soroban-sdk` with the `testutils` feature. They need no network access.

## Continuous integration

CI runs these steps on every push to `main` and on every pull request:

| Step | Command |
|---|---|
| Format | `cargo fmt --all -- --check` |
| Release build | `stellar contract build --locked`, using a checksum-verified Stellar CLI 28.0.0. WASM hashes are published in the job summary. |
| Lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Test | `cargo test --workspace --locked` |

## Test suites

| Suite | Location | Tests |
|---|---|---|
| KYB Attestation Registry | [`kyb-attestation-registry/src/test.rs`](../kyb-attestation-registry/src/test.rs) | 28 |
| Insurance Record | [`insurance-record/src/test.rs`](../insurance-record/src/test.rs) | 27 |
| Upgrade from 0.2.0 | [`upgrade-tests/tests/upgrade.rs`](../upgrade-tests/tests/upgrade.rs) | 2 |

### Coverage

| Area | What is tested |
|---|---|
| Authorization | Every write and administrative function rejects a wrong `admin` argument and missing authorization. The exact authorization payload is asserted. |
| Admin transfer | Proposal, acceptance, cancellation and replacement of a nomination; the previous admin loses access |
| Validation | Expiry must be in the future, the decision vocabulary, non-zero hashes |
| Record rules | Sticky revocation, revocation before the first write, reinstatement, deal binding, payout preconditions, finality after payout |
| Idempotency | Identical replays of writes, revocations, decisions and receipts |
| Validity | `is_valid` at the expiry boundary, and for revoked and unknown subjects |
| Events | The exact topics and data of every event type |
| Storage lifetime | Writes extend records and the instance; extension needs no authorization |
| Upgrade | The released 0.2.0 WASM, pinned by SHA-256 in [`upgrade-tests/fixtures/`](../upgrade-tests/fixtures/), writes state and is upgraded in place to the current build. The tests then check the state and the new rules. |

### Mutation testing

Mutation testing confirms that the suite catches regressions. Each security check is removed or inverted, one at a time, and the suite is rerun. For 0.3.0, all 30 mutations made the suite fail. They covered every authorization check, validation rule and integrity rule. Repeat this when you add a check.

### Network rehearsal

Before a release, rehearse on testnet with throwaway keys:

1. Deploy the previous version.
2. Upgrade it in place.
3. Exercise the rules and events against the live network.

## Adding tests

- Put unit tests in the contract's `src/test.rs`. Use the `try_` client methods to assert contract errors.
- Assert events with `env.events().all()` immediately after the call under test.
- When a release changes behavior, add its WASM to `upgrade-tests/fixtures/<version>/` and extend the upgrade tests.

## Dependency note

`Cargo.lock` keeps `ed25519-dalek` at 2.x, because version 3.x is incompatible with the test utilities of `soroban-env-host` 22. After updating dependencies, re-pin it:

```bash
cargo update -p ed25519-dalek --precise 2.2.0
```

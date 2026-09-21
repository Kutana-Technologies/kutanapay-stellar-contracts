# Contract test plan

`cargo test` against current `soroban-sdk` 22 + crates.io `testutils` hits a `rand_core` split in `soroban-env-host` and does not compile. Until that is remediationsed (SDK bump or a locked `testutils` graph), run:

```bash
stellar contract build
# or
cargo build --release --target wasm32v1-none --workspace
```

## Cases to automate (unauthorized writer, revoke, round-trip)

These match the SCF Tranche 2 suite: unauthorized writer rejection, expiry/revocation handling.

**KYB**

- Constructor stores admin; `get_admin` returns it.
- Admin `write_attestation` then `get_attestation` returns tier, expiry, `revoked=false`.
- Admin `revoke_attestation` sets `revoked=true`.
- Non-admin write panics `not admin`.
- Rewrite after revoke clears `revoked` and updates tier.
- Missing subject returns `None`.
- Expiry is stored but **not** filtered by `get_attestation` — reader tests belong in the Python gate.

**Insurance**

- Admin `record_policy` round-trip on `get_policy`.
- `record_claim_decision` then `record_payout_receipt` keeps decision and sets receipt.
- Non-admin `record_policy` panics `not admin`.
- Missing policy/claim returns `None`.

On-chain verification of a live id: `stellar contract info interface --network testnet --id C…`.

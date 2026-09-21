# Security notes

These contracts are **registries**, not vaults. They never hold USDC. Escrow risk sits with Trustless Work's audited contracts.

## Authorization

- Admin is set once in `__constructor`.
- Every write and `upgrade` calls `require_auth` on that admin.
- A non-admin address panics with `not admin` even if the caller can sign.

KutanaPay production wires that admin to the Signing Service. An outsider deploy uses whatever `G…` they pass at deploy.

## Upgrade

`upgrade(new_wasm_hash)` replaces WASM in place (same `C…`, same storage). There is no timelock or two-key scheme in v0.2. Treat the admin key as high value. SDF Audit Bank review (authorization, upgrade posture, denial/griefing) is planned before mainnet general availability.

## Denial / griefing

- KYB: admin can overwrite or revoke any subject. There is no per-subject self-service.
- Insurance: `record_payout_receipt` can insert a stub claim if none exists. Writers should require an existing decision row off-chain.
- Reads are free of auth; spam is limited to whoever can pay to write (admin only).

## Expiry

KYB `expires_at` is stored but **not** enforced inside `get_attestation`. Enforcing expiry on-chain would be a v0.3 behavior change (new WASM + `upgrade`). Until then, every reader must filter.

## Tests in this repo

`cargo test --workspace` covers admin-only writes, revoke, policy/claim/payout round-trip, and missing-key reads. It does not replace an Audit Bank review.

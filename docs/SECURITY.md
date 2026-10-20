# Security Model

This document describes the trust assumptions, controls and known limitations of version `0.2.0`. To report a vulnerability, see [SECURITY.md](../SECURITY.md) in the repository root.

## Summary

| Property | v0.2.0 |
|---|---|
| Holds or moves funds | No |
| Who can write | One admin address, fixed at deployment |
| Who can read | Anyone |
| Upgradeable | Yes, by the admin, immediately |
| Admin rotation | Only by upgrading |
| On-chain history and events | Not emitted; transaction history only |
| Personal data on-chain | None by design: only account addresses, short codes, timestamps and SHA-256 digests |
| Independent audit | Planned before mainnet |

## Trust assumptions

Users of these contracts trust that:

1. **The admin key is controlled by KutanaPay and kept secure.** The admin can write, overwrite or revoke any record, and can replace the contract code.
2. **The admin writes accurate records.** The contracts make records public and attributable. They do not judge whether a record is true.
3. **Consumers apply the documented rules**, in particular the KYB [validity rule](INTERFACES.md#validity-rule).

## Authorization

- The admin is set once, in `__constructor` at deployment.
- Every write function and `upgrade` checks that the caller-supplied `admin` matches the stored admin, then calls `require_auth` on it.
- A different address is rejected with `not admin`. The correct address without its signature is rejected with `Error(Auth, InvalidAction)`.
- Both paths are covered by tests, and these checks pass. See [TESTING.md](TESTING.md).

## Upgrades

`upgrade(new_wasm_hash)` replaces the contract code in a single admin-authorized transaction. The contract ID and all storage are kept. There is no timelock or multi-party approval inside the contract.

Because an upgrade can change any behavior, **the admin key is the most important control in the system**. Anyone can see an upgrade on-chain: the contract's WASM hash changes, and the new code can be checked using [VERIFY.md](VERIFY.md).

## Admin key management

Recommended practice for any deployment:

- Use a dedicated admin account that is not used for any other role.
- Make the admin a [Stellar multisig account](https://developers.stellar.org/docs/learn/encyclopedia/security/signatures-multisig) with a threshold of two or more signers, held on separate hardware or KMS-backed keys. This needs no contract change: `require_auth` enforces the account's thresholds.
- Keep the deployment and admin keys out of source control and CI.

## Known limitations

These are behaviors of v0.2.0 that integrators must account for. Planned changes are listed under [Roadmap](#roadmap).

| Area | Limitation | Mitigation in v0.2.0 |
|---|---|---|
| Governance | Single admin; upgrades take effect immediately; no rotation function | Multisig admin account; on-chain monitoring of WASM hash changes |
| KYB expiry | `get_attestation` does not check expiry or revocation | Consumers apply the validity rule |
| KYB reinstatement | Writing an attestation over a revoked one clears the revocation | Writers re-check the latest KYB decision before each write |
| Record history | Writes overwrite earlier values; no events are emitted | Use transaction history (Horizon, RPC, block explorers) for an audit trail |
| Payout without decision | `record_payout_receipt` creates a placeholder claim (`decision = "paid"`, zero `decision_hash`) if no claim exists | Writers record a decision first; verifiers ignore zero-hash decisions |
| Claim linkage | Claim writes set `deal_id` from the latest call | Writers use one `deal_id` per claim |
| Storage lifetime | TTLs are not extended by the contract | Operators extend TTLs ([INTEGRATION.md](INTEGRATION.md#operational-requirements)) |
| Errors | Errors are untyped panics | Clients match on the documented error text |

## Denial of service

All writes need admin authorization, so third parties cannot add records or fill up storage. Reads need no authorization.

## Roadmap

The following changes are planned for a future version and will be released through `upgrade`, with the new WASM published and verifiable:

- Two-step admin transfer and a delay on upgrades.
- Contract events for every state change.
- An on-chain validity check for KYB attestations.
- Explicit reinstatement after revocation.
- Consistency checks between claims, decisions and payouts.
- Automatic TTL extension.
- Typed contract errors.

## Audit status

These contracts have not yet been audited by an independent party. An independent security review is planned before any mainnet deployment. Its report will be linked here.

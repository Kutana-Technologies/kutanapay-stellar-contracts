# Security Model

This document describes what the contracts guarantee, what they assume, and how they are protected. To report a vulnerability, follow the [security policy](../SECURITY.md).

## Scope

The contracts are **registries**. They hold no assets and make no external calls. Their purpose is to make records public, attributable and tamper-evident.

## Trust assumptions

| Assumption | Why it is needed |
|---|---|
| The operator keeps the admin key secure | The admin is the only writer and can upgrade the contract |
| The admin records accurate information | The contracts guarantee integrity and provenance, not that a record is true |
| Readers use the documented checks | For example, `is_valid` for KYB standing |

## Guarantees

### Access control

- Only the stored admin can create, change or revoke records. Each write checks the supplied `admin` against the stored value, then requires that address's authorization.
- Only the stored admin can nominate a successor, cancel a nomination, or upgrade the contract.
- Only the nominee can accept the admin role, so the role cannot pass to an address that nobody controls.
- Read and storage-extension functions cannot change any record.

### Record integrity

| Guarantee | Enforced by |
|---|---|
| A revocation persists until it is explicitly reinstated | `write_attestation` rejects revoked subjects |
| Writes that arrive out of order cannot bypass a revocation | Revoking an unknown subject records it as revoked |
| KYB standing is never written already expired | `expires_at` must be in the future |
| Every payout is backed by an approved decision for the same deal | `record_payout_receipt` checks the claim |
| Paid claims are final | The decision and receipt cannot change after payout |
| A claim belongs to exactly one deal | `deal_id` cannot change |
| Document commitments are meaningful | All-zero hashes are rejected |
| Retries cannot corrupt state | Identical replays change nothing |

### Transparency

- Every state change, admin change and upgrade emits an event.
- Release builds are reproducible. Anyone can confirm that the deployed code matches a source commit; see [VERIFY.md](VERIFY.md).

### Privacy

On-chain data is limited to account addresses, short codes, timestamps and 32-byte digests. Integrators must not use personal data as identifiers or as guessable hash preimages; see [INTEGRATION.md](INTEGRATION.md#1-derive-identifiers).

### Availability

- Third parties cannot add records, so they cannot fill contract storage.
- Writes keep their own storage alive. Anyone can pay to extend storage lifetime, and archived entries can be restored.

## Operational security

| Control | Practice |
|---|---|
| Admin key | A dedicated multisig account with independent signers; see [DEPLOYMENT.md](DEPLOYMENT.md#choose-the-admin) |
| Admin rotation | Two-step transfer; cancel nominations that are no longer wanted |
| Upgrades | Built reproducibly, verified before and after, and announced by the `upgraded` event |
| Monitoring | Alert on `upgraded`, `adm_prop`, `adm_set` and `adm_cncl` events |

## Assurance

| Activity | Status |
|---|---|
| Unit tests for every authorization path, validation rule and event | Run in CI; see [TESTING.md](TESTING.md) |
| Upgrade tests from the previous release | Run in CI |
| Mutation testing of the security checks | Performed for 0.3.0 |
| Independent audit | Planned before mainnet; the report will be linked here |

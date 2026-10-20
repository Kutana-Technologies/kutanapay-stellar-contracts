# Integration Guide

This guide describes how KutanaPay Trade Secure uses these contracts alongside Trustless Work escrow, and how an independent party can reproduce the on-chain identifiers.

## Architecture

```
Trade Secure (off-chain)
  │  Each deal event is written to a transactional outbox
  │  (one row per event, retried until confirmed on-chain)
  │
  ├── Deal accepted ─────────────► Trustless Work: deploy multi-release escrow
  ├── Deal funded ───────────────► Trustless Work: fund escrow (USDC)
  ├── Delivery confirmed ────────► SEP-53 signed attestation (off-chain)
  │                                 └─► Trustless Work: approve_milestone
  ├── Milestone released ────────► Trustless Work: release
  ├── Business KYB approved ─────► KYB Registry:     write_attestation
  ├── Business KYB rejected ─────► KYB Registry:     revoke_attestation
  ├── Insurance policy issued ───► Insurance Record: record_policy
  ├── Claim decided ─────────────► Insurance Record: record_claim_decision
  └── Claim paid ────────────────► Insurance Record: record_payout_receipt
```

Funds only ever move through Trustless Work escrow. A defect in the contracts in this repository can produce an incorrect standing or record. It cannot move or lock funds.

## Trustless Work escrow (external)

KutanaPay does not deploy a custom escrow contract. When a deal is accepted, the platform deploys a Trustless Work multi-release escrow with fixed roles and USDC as the asset.

| Deal event | Trustless Work operation | Role |
|---|---|---|
| Deal accepted | Deploy escrow | Platform |
| Buyer payment settled | `fund_escrow` | Treasury |
| Buyer confirms delivery | `approve_milestone(index)` | Approver |
| Milestone release | `release` | Release signer |
| Dispute | Dispute flags | Dispute resolver |

Delivery confirmation produces a SEP-53 signed message, which gates the approver's `approve_milestone` call. The digest is part of the on-chain transaction only if it is also supplied as milestone evidence. Shipment events do not produce an attestation.

## KYB Attestation Registry

1. A business completes KYB review and is approved.
2. The platform assigns the business a Stellar account (`G…`) if it doesn't already have one.
3. The platform calls `write_attestation(admin, subject, tier, expires_at)`, where `subject` is that account. `expires_at` is the review expiry in Unix seconds.
4. When the KYB gate is enabled, a deal can only be created if both counterparties are in good standing under the [validity rule](INTERFACES.md#validity-rule).
5. If approval is withdrawn, the platform calls `revoke_attestation(admin, subject)`.

The `subject` is always a Stellar account address. Internal user or business identifiers are never written.

## Insurance Record

| Trigger | Call |
|---|---|
| Insurance review approved | `record_policy(admin, deal_id, escrow_id, policy_hash, status)` |
| Claim decided | `record_claim_decision(admin, deal_id, claim_id, decision, decision_hash)` |
| Claim paid | `record_payout_receipt(admin, deal_id, claim_id, receipt_hash)` |

Writers should record a claim decision before its payout receipt. See the note in [INTERFACES.md](INTERFACES.md#insurance-record).

## Identifier and hash encoding

Every `BytesN<32>` argument is derived from an off-chain value as follows:

| Argument | Source value | Encoding |
|---|---|---|
| `deal_id` | Trade Secure deal ID (UUID string) | `SHA-256(UTF-8(id))` |
| `claim_id` | Claim ID (UUID string) | `SHA-256(UTF-8(id))` |
| `escrow_id` | Trustless Work escrow contract address (`C…`), or `no-escrow` if none | `SHA-256(UTF-8(value))` |
| `policy_hash`, `decision_hash`, `receipt_hash` | KutanaPay's canonical record for the policy, decision or payout receipt | `SHA-256` of that record (see below) |

Rules:

- The ID string is used exactly as stored, after trimming surrounding whitespace. UUIDs use the canonical lowercase hyphenated form, for example `3f2b6c1e-8a4d-4c2b-9e7f-1a2b3c4d5e6f`.
- If a value is already 64 hexadecimal characters, it is treated as a precomputed digest and decoded directly rather than hashed again.

Example:

```bash
printf '%s' '3f2b6c1e-8a4d-4c2b-9e7f-1a2b3c4d5e6f' | sha256sum   # deal_id
```

**Document hashes.** The current Trade Secure writer computes `policy_hash`, `decision_hash` and `receipt_hash` over KutanaPay's canonical record references, not over document file bytes. The on-chain value is therefore a commitment that shows whether a record changed after it was anchored. A third party can confirm it only with the preimage supplied by KutanaPay. Hashing document file bytes directly, so that anyone holding the file can verify it, is planned.

## Operational requirements

**Storage lifetime.** Soroban storage entries have a time-to-live (TTL). Version 0.2.0 does not extend TTLs itself. Entries that are not extended are archived when their TTL lapses, and that includes the contract instance. Since Protocol 23, an invocation that touches an archived entry restores it automatically at extra fee. Some read paths, such as RPC `getLedgerEntries`, do not return archived entries. Operators should extend TTLs regularly. For example, to extend the contract instance and its code:

```bash
stellar contract extend --id <CONTRACT_ID> --durability persistent \
  --ledgers-to-extend 500000 --network testnet --source-account <payer>
```

Individual records can be extended the same way by passing their ledger key with `--key-xdr`.

Readers should not interpret a missing entry from a direct ledger read as "no record" without simulating a contract call.

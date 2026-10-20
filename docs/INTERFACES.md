# Contract Interfaces

This document specifies the public interface of each contract at version `0.2.0`. The machine-readable specification (SEP-48) is embedded in each WASM and can be printed with:

```bash
stellar contract info interface --network testnet --contract-id <CONTRACT_ID>
stellar contract info interface --wasm target/wasm32v1-none/release/<name>.wasm
```

## Conventions

| Type | Meaning |
|---|---|
| `Address` | A Stellar account (`G…`) or contract (`C…`) address |
| `Symbol` | A short code: 1 to 32 characters drawn from `a-z A-Z 0-9 _` |
| `BytesN<32>` | 32 bytes, usually a SHA-256 digest. The CLI accepts it as 64 hex characters. |
| `u64` timestamps | Unix time in seconds (UTC) |

**Authorization.** Every write function takes an `admin` argument. The call succeeds only if that argument equals the stored admin address **and** the transaction carries that address's authorization (`require_auth`). If the address doesn't match, the call fails with `not admin`. If the address matches but the signature is missing, it fails with `Error(Auth, InvalidAction)`. Read functions need no authorization.

**Write semantics.** All writes are upserts: they create the record or replace it. The contracts keep no history and emit no events. Each transaction's effects remain visible in the ledger's transaction history.

## KYB Attestation Registry

Records whether a business's Stellar account has passed KYB review, at which tier, and until when.

### Functions

| Function | Arguments | Auth | Returns | Behavior |
|---|---|---|---|---|
| `__constructor` | `admin: Address` | deployment | — | Stores the admin. Runs once, at deployment. |
| `write_attestation` | `admin: Address`, `subject: Address`, `tier: Symbol`, `expires_at: u64` | admin | — | Creates or replaces the subject's attestation with `revoked = false` |
| `revoke_attestation` | `admin: Address`, `subject: Address` | admin | — | Sets `revoked = true`. Does nothing if the subject has no attestation. |
| `get_attestation` | `subject: Address` | none | `Option<Attestation>` | Returns the stored row, or `None` |
| `get_admin` | — | none | `Address` | Returns the admin |
| `upgrade` | `new_wasm_hash: BytesN<32>` | admin | — | Replaces the contract code. The contract ID and storage are kept. |

### Types

```rust
pub struct Attestation {
    pub tier: Symbol,     // for example "standard"
    pub expires_at: u64,  // Unix seconds
    pub revoked: bool,
}
```

### Validity rule

`get_attestation` returns the stored row as-is. It does **not** check expiry or revocation. A subject is in good standing only if **all** of the following hold:

1. `get_attestation` returns a row.
2. `revoked == false`.
3. `expires_at` is later than the current time.

Every consumer must apply this rule.

Writing an attestation for a subject that was previously revoked replaces the row and sets `revoked = false`.

### Storage

| Key | Storage | Value |
|---|---|---|
| `DataKey::Admin` | instance | `Address` |
| `("attest", subject: Address)` | persistent | `Attestation` |

## Insurance Record

Anchors trade-insurance documents to a deal, so that anyone holding a document can confirm it is the one recorded.

### Functions

| Function | Arguments | Auth | Returns | Behavior |
|---|---|---|---|---|
| `__constructor` | `admin: Address` | deployment | — | Stores the admin |
| `record_policy` | `admin`, `deal_id: BytesN<32>`, `escrow_id: BytesN<32>`, `policy_hash: BytesN<32>`, `status: Symbol` | admin | — | Creates or replaces the policy record for `deal_id` |
| `record_claim_decision` | `admin`, `deal_id: BytesN<32>`, `claim_id: BytesN<32>`, `decision: Symbol`, `decision_hash: BytesN<32>` | admin | — | Creates or replaces the claim record for `claim_id`. Keeps any existing `receipt_hash`. |
| `record_payout_receipt` | `admin`, `deal_id: BytesN<32>`, `claim_id: BytesN<32>`, `receipt_hash: BytesN<32>` | admin | — | Sets `receipt_hash` and `deal_id` on the claim. See note. |
| `get_policy` | `deal_id: BytesN<32>` | none | `Option<PolicyRecord>` | |
| `get_claim` | `claim_id: BytesN<32>` | none | `Option<ClaimRecord>` | |
| `get_admin` | — | none | `Address` | |
| `upgrade` | `new_wasm_hash: BytesN<32>` | admin | — | Replaces the contract code. The contract ID and storage are kept. |

> [!NOTE]
> If no claim record exists, `record_payout_receipt` creates one with `decision = "paid"` and an all-zero `decision_hash`. A zero `decision_hash` therefore means no claim decision was recorded on-chain. Verifiers should not treat such a record as a decision.

### Types

```rust
pub struct PolicyRecord {
    pub escrow_id: BytesN<32>,
    pub policy_hash: BytesN<32>,
    pub status: Symbol,                 // for example "issued"
}

pub struct ClaimRecord {
    pub deal_id: BytesN<32>,
    pub decision: Symbol,               // for example "approved", "rejected"
    pub decision_hash: BytesN<32>,
    pub receipt_hash: Option<BytesN<32>>,
}
```

For how `deal_id`, `claim_id`, `escrow_id` and the document hashes are derived, see [Identifier and hash encoding](INTEGRATION.md#identifier-and-hash-encoding).

### Storage

| Key | Storage | Value |
|---|---|---|
| `DataKey::Admin` | instance | `Address` |
| `("policy", deal_id)` | persistent | `PolicyRecord` |
| `("claim", claim_id)` | persistent | `ClaimRecord` |

## Embedded metadata (SEP-46)

```bash
stellar contract info meta --network testnet --contract-id <CONTRACT_ID>
```

| Key | KYB Attestation Registry | Insurance Record |
|---|---|---|
| `name` | `KutanaPay KYB Attestation Registry` | `KutanaPay Insurance Record` |
| `org` | `KutanaPay` | `KutanaPay` |
| `binver` | `0.2.0` | `0.2.0` |
| `desc` | `On-chain KYB standing: tier, expiry, revocation. No PII.` | `On-chain insurance policy/claim/payout hashes keyed to deals. No PII.` |

The toolchain also embeds `rsver` (Rust version), `rssdkver` (Soroban SDK version and commit) and `cliver` (Stellar CLI version). These are useful when reproducing a build.

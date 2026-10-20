# Contract Reference

This is the complete public interface of both contracts at version `0.3.0`. For the concepts behind it, read the [overview](OVERVIEW.md) first.

The machine-readable specification (SEP-48) is embedded in each WASM:

```bash
stellar contract info interface --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm
stellar contract info interface --network testnet --contract-id <CONTRACT_ID>
```

## Conventions

### Types

| Type | Meaning |
|---|---|
| `Address` | A Stellar account (`G…`) or contract (`C…`) |
| `Symbol` | A short code of 1–32 characters from `a-z A-Z 0-9 _` |
| `BytesN<32>` | 32 bytes, typically a SHA-256 digest. The CLI accepts it as 64 hex characters. |
| `u64` time | Unix seconds, compared with the ledger close time |

### Authorization

| Marker | Requirement |
|---|---|
| **admin** | The `admin` argument must equal the stored admin, and the transaction must carry that address's authorization |
| **admin (stored)** | The stored admin's authorization. The function takes no `admin` argument. |
| **nominee** | The pending admin's authorization |
| **none** | No authorization |

If the `admin` argument is wrong, the call fails with the contract error `NotAdmin`. If the required authorization is missing, the call fails with the host error `Error(Auth, InvalidAction)`.

### Errors, replays and events

- **Errors** are returned as `Error(Contract, #<code>)`. They mean the request is not valid in the contract's current state, so retrying the identical call returns the same error.
- **Replays:** writing an identical record that is already stored succeeds. State does not change and no event is emitted.
- **Events:** every state change publishes one event. Each contract's events are listed below.

## KYB Attestation Registry

### Functions

#### Records

| Function | Auth | Description |
|---|---|---|
| `write_attestation(admin, subject: Address, tier: Symbol, expires_at: u64) -> Result<(), Error>` | admin | Creates or updates an active attestation. Fails if `expires_at` is not in the future or the subject is revoked. |
| `reinstate_attestation(admin, subject: Address, tier: Symbol, expires_at: u64) -> Result<(), Error>` | admin | Stores an active attestation whether or not the subject is revoked. Fails if `expires_at` is not in the future. |
| `revoke_attestation(admin, subject: Address) -> Result<(), Error>` | admin | Marks the subject revoked. If the subject has no attestation, stores `{ tier: "none", expires_at: 0, revoked: true }`. |
| `get_attestation(subject: Address) -> Option<Attestation>` | none | Returns the stored attestation as-is |
| `is_valid(subject: Address) -> bool` | none | Returns `true` if an attestation exists, `revoked` is `false`, and `expires_at` is later than the ledger time |

#### Administration

| Function | Auth | Description |
|---|---|---|
| `__constructor(admin: Address)` | deployment | Sets the admin |
| `get_admin() -> Address` | none | Returns the current admin |
| `get_pending_admin() -> Option<Address>` | none | Returns the nominated successor, if any |
| `propose_admin(new_admin: Address)` | admin (stored) | Nominates a successor, replacing any earlier nomination |
| `accept_admin() -> Result<(), Error>` | nominee | Makes the nominee the admin |
| `cancel_admin_transfer() -> Result<(), Error>` | admin (stored) | Withdraws the nomination |
| `upgrade(new_wasm_hash: BytesN<32>)` | admin (stored) | Replaces the contract code. The contract ID and storage are kept. |

#### Storage lifetime

| Function | Auth | Description |
|---|---|---|
| `extend_ttl()` | none | Extends the lifetime of the contract instance and its code |
| `extend_attestation_ttl(subject: Address)` | none | Extends the lifetime of one attestation. Does nothing if the subject has none. |

### Types

```rust
pub struct Attestation {
    pub tier: Symbol,
    pub expires_at: u64,
    pub revoked: bool,
}
```

### Errors

| Code | Name | Returned when |
|---|---|---|
| 1 | `NotAdmin` | `admin` is not the stored admin |
| 2 | `ExpiryNotInFuture` | `expires_at` ≤ ledger time |
| 3 | `AttestationRevoked` | `write_attestation` is called for a revoked subject |
| 4 | `NoPendingAdmin` | `accept_admin` or `cancel_admin_transfer` is called with no nomination |

### Events

| Topics | Data | Emitted by |
|---|---|---|
| `("written", subject)` | `(tier, expires_at)` | `write_attestation` |
| `("reinstate", subject)` | `(tier, expires_at)` | `reinstate_attestation` |
| `("revoked", subject)` | `()` | `revoke_attestation` |
| `("adm_prop",)` | `new_admin` | `propose_admin` |
| `("adm_set",)` | `(previous_admin, new_admin)` | `accept_admin` |
| `("adm_cncl",)` | `withdrawn_nominee` | `cancel_admin_transfer` |
| `("upgraded",)` | `new_wasm_hash` | `upgrade` |

### Storage

| Key | Durability | Value |
|---|---|---|
| `DataKey::Admin` | instance | `Address` |
| `DataKey::PendingAdmin` | instance | `Address` |
| `("attest", subject)` | persistent | `Attestation` |

## Insurance Record

### Functions

#### Records

| Function | Auth | Description |
|---|---|---|
| `record_policy(admin, deal_id: BytesN<32>, escrow_id: BytesN<32>, policy_hash: BytesN<32>, status: Symbol) -> Result<(), Error>` | admin | Creates or updates the policy for `deal_id` |
| `record_claim_decision(admin, deal_id: BytesN<32>, claim_id: BytesN<32>, decision: Symbol, decision_hash: BytesN<32>) -> Result<(), Error>` | admin | Records `approved` or `denied` for a claim. The decision can change until a payout is recorded. |
| `record_payout_receipt(admin, deal_id: BytesN<32>, claim_id: BytesN<32>, receipt_hash: BytesN<32>) -> Result<(), Error>` | admin | Attaches a receipt to an approved claim with the same `deal_id` |
| `get_policy(deal_id: BytesN<32>) -> Option<PolicyRecord>` | none | Returns the stored policy, if any |
| `get_claim(claim_id: BytesN<32>) -> Option<ClaimRecord>` | none | Returns the stored claim, if any |

Validation rules:

| Rule | Error if broken |
|---|---|
| `decision` must be `approved` or `denied` | `InvalidDecision` |
| `policy_hash`, `decision_hash` and `receipt_hash` must not be all zeros | `InvalidHash` |
| A claim's `deal_id` cannot change | `DealMismatch` |
| A payout needs a recorded decision | `ClaimNotFound` |
| That decision must be `approved` | `ClaimNotApproved` |
| The decision cannot change after payout | `DecisionFinal` |
| The receipt cannot change after payout | `ReceiptAlreadyRecorded` |

#### Administration and storage lifetime

These functions behave exactly as in the KYB registry: `__constructor`, `get_admin`, `get_pending_admin`, `propose_admin`, `accept_admin`, `cancel_admin_transfer`, `upgrade` and `extend_ttl`. To extend a single record, use `extend_policy_ttl(deal_id)` or `extend_claim_ttl(claim_id)`.

### Types

```rust
pub struct PolicyRecord {
    pub escrow_id: BytesN<32>,
    pub policy_hash: BytesN<32>,
    pub status: Symbol,                   // for example "issued"
}

pub struct ClaimRecord {
    pub deal_id: BytesN<32>,
    pub decision: Symbol,                 // "approved" or "denied"
    pub decision_hash: BytesN<32>,
    pub receipt_hash: Option<BytesN<32>>,
}
```

### Errors

| Code | Name | Returned when |
|---|---|---|
| 1 | `NotAdmin` | `admin` is not the stored admin |
| 2 | `InvalidDecision` | `decision` is not `approved` or `denied` |
| 3 | `ClaimNotFound` | A payout is attempted with no recorded decision |
| 4 | `DealMismatch` | `deal_id` differs from the claim's recorded deal |
| 5 | `ClaimNotApproved` | A payout is attempted on a claim that is not approved |
| 6 | `DecisionFinal` | A different decision is attempted after payout |
| 7 | `ReceiptAlreadyRecorded` | A different receipt is attempted after payout |
| 8 | `NoPendingAdmin` | `accept_admin` or `cancel_admin_transfer` is called with no nomination |
| 9 | `InvalidHash` | A document hash is all zeros |

### Events

| Topics | Data | Emitted by |
|---|---|---|
| `("policy", deal_id)` | `PolicyRecord` | `record_policy` |
| `("decision", claim_id)` | `(decision, decision_hash)` | `record_claim_decision` |
| `("payout", claim_id)` | `receipt_hash` | `record_payout_receipt` |
| `("adm_prop",)`, `("adm_set",)`, `("adm_cncl",)`, `("upgraded",)` | Same as the KYB registry | Administration functions |

### Storage

| Key | Durability | Value |
|---|---|---|
| `DataKey::Admin`, `DataKey::PendingAdmin` | instance | `Address` |
| `("policy", deal_id)` | persistent | `PolicyRecord` |
| `("claim", claim_id)` | persistent | `ClaimRecord` |

## Metadata (SEP-46)

```bash
stellar contract info meta --network testnet --contract-id <CONTRACT_ID>
```

| Key | KYB Attestation Registry | Insurance Record |
|---|---|---|
| `name` | `KutanaPay KYB Attestation Registry` | `KutanaPay Insurance Record` |
| `org` | `KutanaPay` | `KutanaPay` |
| `binver` | `0.3.0` | `0.3.0` |
| `desc` | `On-chain KYB standing: tier, expiry, revocation. No PII.` | `On-chain insurance policy/claim/payout hashes keyed to deals. No PII.` |

The toolchain also embeds `rsver` (Rust version), `rssdkver` (Soroban SDK version and commit) and `cliver` (Stellar CLI version).

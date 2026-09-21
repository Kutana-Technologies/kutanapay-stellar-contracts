# Contract interfaces (SEP-46 / SEP-48)

Both crates embed SEP-46 metadata in the WASM via `contractmeta!`. Anyone can recover it from a built WASM or a live `C…` id.

```bash
stellar contract info meta --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm
stellar contract info interface --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm
stellar contract info interface --network testnet --id <CONTRACT_ID>
```

`binver` is `0.2.0`. `org` is `KutanaPay`.

## KYB Attestation Registry

SEP-46: `name=KutanaPay KYB Attestation Registry`, `desc=On-chain KYB standing: tier, expiry, revocation. No PII.`

| Function | Args | Auth | Effect |
|----------|------|------|--------|
| `__constructor` | `admin: Address` | deploy only | Stores sole admin |
| `write_attestation` | `admin`, `subject`, `tier: Symbol`, `expires_at: u64` | stored admin + `require_auth` | Upsert standing; `revoked=false` |
| `revoke_attestation` | `admin`, `subject` | stored admin + `require_auth` | Sets `revoked=true` if a row exists |
| `get_attestation` | `subject` | none | `Option<Attestation>` |
| `get_admin` | — | none | Admin address |
| `upgrade` | `new_wasm_hash: BytesN<32>` | stored admin + `require_auth` | Replace WASM, keep `C…` and storage |

`Attestation { tier, expires_at, revoked }`.

**Application-layer expiry.** `get_attestation` returns the stored row even if `expires_at` is in the past or `revoked` is true. KutanaPay's deal gate treats a row as valid only when `revoked=false` and `expires_at` is in the future. Callers must apply the same rule.

## Insurance Record

SEP-46: `name=KutanaPay Insurance Record`, `desc=On-chain insurance policy/claim/payout hashes keyed to deals. No PII.`

Ids and hashes are `BytesN<32>` (SHA-256 of off-chain UUIDs or document bytes — see [INTEGRATION.md](INTEGRATION.md)).

| Function | Args | Auth | Effect |
|----------|------|------|--------|
| `__constructor` | `admin: Address` | deploy only | Stores sole admin |
| `record_policy` | `admin`, `deal_id`, `escrow_id`, `policy_hash`, `status` | stored admin | Upsert policy for `deal_id` |
| `record_claim_decision` | `admin`, `deal_id`, `claim_id`, `decision`, `decision_hash` | stored admin | Upsert claim; keeps existing `receipt_hash` |
| `record_payout_receipt` | `admin`, `deal_id`, `claim_id`, `receipt_hash` | stored admin | Sets receipt; creates a stub claim if none exists |
| `get_policy` | `deal_id` | none | `Option<PolicyRecord>` |
| `get_claim` | `claim_id` | none | `Option<ClaimRecord>` |
| `get_admin` | — | none | Admin address |
| `upgrade` | `new_wasm_hash` | stored admin | Replace WASM in place |

`PolicyRecord { escrow_id, policy_hash, status }`  
`ClaimRecord { deal_id, decision, decision_hash, receipt_hash }`

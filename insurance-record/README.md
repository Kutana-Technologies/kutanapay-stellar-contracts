# Insurance Record

A Soroban contract that anchors trade-insurance documents to a Trade Secure deal. It records SHA-256 digests of the policy, claim decision and payout receipt, so anyone holding a document can confirm it matches the recorded one. It stores no documents and no personal data.

| | |
|---|---|
| Version | `0.2.0` |
| Testnet ID | `CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS` |
| Writes | Admin only |
| Reads | Public |

## Interface

| Function | Description |
|---|---|
| `record_policy(admin, deal_id, escrow_id, policy_hash, status)` | Create or replace the policy record for a deal |
| `record_claim_decision(admin, deal_id, claim_id, decision, decision_hash)` | Create or replace a claim decision |
| `record_payout_receipt(admin, deal_id, claim_id, receipt_hash)` | Attach a payout receipt to a claim |
| `get_policy(deal_id) -> Option<PolicyRecord>` | Read a policy record |
| `get_claim(claim_id) -> Option<ClaimRecord>` | Read a claim record |
| `get_admin() -> Address` | Read the admin address |
| `upgrade(new_wasm_hash)` | Replace the contract code (admin only) |

```rust
pub struct PolicyRecord { pub escrow_id: BytesN<32>, pub policy_hash: BytesN<32>, pub status: Symbol }
pub struct ClaimRecord {
    pub deal_id: BytesN<32>,
    pub decision: Symbol,
    pub decision_hash: BytesN<32>,
    pub receipt_hash: Option<BytesN<32>>,
}
```

All identifiers are SHA-256 digests. See [Identifier and hash encoding](../docs/INTEGRATION.md#identifier-and-hash-encoding).

## More

- [Full interface specification](../docs/INTERFACES.md#insurance-record)
- [Security model and known limitations](../docs/SECURITY.md)
- [How to verify a record on-chain](../docs/VERIFY.md#4-check-an-insurance-record)

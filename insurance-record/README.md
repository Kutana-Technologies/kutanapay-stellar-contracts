# Insurance Record

A Soroban contract that anchors SHA-256 commitments to an insurance policy, a claim decision and a payout receipt. Records are linked by 32-byte identifiers. The contract stores no documents and no personal data.

## Usage

```bash
# Read a claim
stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account <any> --send no \
  -- get_claim --claim_id <64-hex>

# Record a decision (admin)
stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account admin \
  -- record_claim_decision --admin <ADMIN> --deal_id <64-hex> --claim_id <64-hex> \
     --decision approved --decision_hash <64-hex>
```

## Interface

| Function | Description |
|---|---|
| `record_policy(admin, deal_id, escrow_id, policy_hash, status)` | Create or update a policy |
| `record_claim_decision(admin, deal_id, claim_id, decision, decision_hash)` | Record `approved` or `denied`; final once paid |
| `record_payout_receipt(admin, deal_id, claim_id, receipt_hash)` | Attach an immutable receipt to an approved claim |
| `get_policy(deal_id)`, `get_claim(claim_id)` | Read records |
| `get_admin()`, `get_pending_admin()` | Read the admin and any nominee |
| `propose_admin`, `accept_admin`, `cancel_admin_transfer` | Two-step admin transfer |
| `upgrade(new_wasm_hash)` | Replace the contract code (admin) |
| `extend_ttl()`, `extend_policy_ttl(deal_id)`, `extend_claim_ttl(claim_id)` | Extend storage lifetime (anyone) |

## Further reading

- [Overview: claim lifecycle](../docs/OVERVIEW.md#insurance-record)
- [Contract reference](../docs/INTERFACES.md#insurance-record): validation rules, errors, events and storage
- [Integration guide](../docs/INTEGRATION.md)

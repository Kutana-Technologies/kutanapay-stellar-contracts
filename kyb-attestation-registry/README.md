# KYB Attestation Registry

A Soroban contract that publishes the know-your-business (KYB) standing of Stellar addresses: a tier, an expiry and a revocation flag. It stores no personal data.

## Usage

```bash
# Is this address in good standing now?
stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account <any> --send no \
  -- is_valid --subject <G-ADDRESS>

# Record standing (admin)
stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account admin \
  -- write_attestation --admin <ADMIN> --subject <G-ADDRESS> --tier standard --expires_at <UNIX>
```

## Interface

| Function | Description |
|---|---|
| `write_attestation(admin, subject, tier, expires_at)` | Create or update an active attestation |
| `reinstate_attestation(admin, subject, tier, expires_at)` | Clear a revocation and store a fresh attestation |
| `revoke_attestation(admin, subject)` | Revoke a subject |
| `is_valid(subject) -> bool` | Whether the subject is in good standing now |
| `get_attestation(subject) -> Option<Attestation>` | Read the stored attestation |
| `get_admin()`, `get_pending_admin()` | Read the admin and any nominee |
| `propose_admin`, `accept_admin`, `cancel_admin_transfer` | Two-step admin transfer |
| `upgrade(new_wasm_hash)` | Replace the contract code (admin only) |
| `extend_ttl()`, `extend_attestation_ttl(subject)` | Extend storage lifetime (anyone) |

## Further reading

- [Overview: attestation lifecycle](../docs/OVERVIEW.md#kyb-attestation-registry)
- [Contract reference: types, errors, events, storage](../docs/INTERFACES.md#kyb-attestation-registry)
- [Integration guide](../docs/INTEGRATION.md)

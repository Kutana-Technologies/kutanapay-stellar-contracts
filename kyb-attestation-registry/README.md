# KYB Attestation Registry

A Soroban contract that publishes the know-your-business (KYB) standing of a business's Stellar account: its tier, its expiry, and whether the standing has been revoked. It stores no personal data.

| | |
|---|---|
| Version | `0.2.0` |
| Testnet ID | `CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG` |
| Writes | Admin only |
| Reads | Public |

## Interface

| Function | Description |
|---|---|
| `write_attestation(admin, subject, tier, expires_at)` | Create or replace a subject's attestation |
| `revoke_attestation(admin, subject)` | Mark a subject's attestation as revoked |
| `get_attestation(subject) -> Option<Attestation>` | Read a subject's attestation |
| `get_admin() -> Address` | Read the admin address |
| `upgrade(new_wasm_hash)` | Replace the contract code (admin only) |

```rust
pub struct Attestation { pub tier: Symbol, pub expires_at: u64, pub revoked: bool }
```

## Checking standing

`get_attestation` returns the stored row without checking it. A subject is in good standing only if the row exists, `revoked` is `false`, and `expires_at` is in the future. See the [validity rule](../docs/INTERFACES.md#validity-rule).

## More

- [Full interface specification](../docs/INTERFACES.md#kyb-attestation-registry)
- [Security model and known limitations](../docs/SECURITY.md)
- [How to verify standing on-chain](../docs/VERIFY.md#3-check-a-businesss-kyb-standing)

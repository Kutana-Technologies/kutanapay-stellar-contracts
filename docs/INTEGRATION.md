# Integration guide — deal events to chain

Architecture-level map of how Trade Secure drives **Trustless Work escrow** and these two registries. Deploying these contracts requires an admin `G…` the deployer controls.

```
Buyer / seller actions on Trade Secure
        │
        ▼
Platform outbox (one row per deal event, idempotent)
        │
        ├── DEAL_ACCEPTED ──────────► Trustless Work: deploy multi-release escrow
        ├── DEAL_FUNDED ────────────► Trustless Work: fund USDC
        ├── DEAL_DELIVERY_CONFIRMED ► SEP-53 sign attestation
        │                              └── DEAL_MILESTONE_APPROVED ► TW approve_milestone
        ├── DEAL_MILESTONE_RELEASED ► Trustless Work: release
        ├── KYC approved (business) ► KYB registry: write_attestation(subject G-address)
        ├── KYC rejected            ► KYB registry: revoke_attestation
        └── Insurance review approved / claim / payout
                                    ► Insurance record: record_policy / record_claim_decision / record_payout_receipt
```

Neither authored contract holds funds. The worst case for a registry bug is a wrong standing or hash, not lost USDC.

## Trustless Work escrow (not in this repo)

KutanaPay does not ship a custom escrow WASM. On deal accept the platform deploys a Trustless Work multi-release contract with frozen roles (approver, service provider, release signer, dispute resolver, platform, receiver) and USDC as the trustline.

| Deal event | TW operation | Typical role |
|------------|--------------|--------------|
| Partner accepts | deploy escrow | deploy / platform |
| Buyer pays (fiat settled, USDC in treasury) | `fund_escrow` | treasury |
| Buyer confirms delivery | SEP-53 attestation, then `approve_milestone(index)` | approver |
| Release | `release` for that milestone | release signer |
| Dispute | TW dispute flags | dispute resolver |

Shipping currently does **not** write an attestation; only delivery confirmation does.

The SEP-53 digest is used as an **off-chain signing gate** for the approver key. The Trustless Work `approve_milestone` invoke itself is `(milestone_index, approver)` — it does not embed the digest unless you also set milestone `newEvidence` via change-status. For public hash-on-chain matching, pass the hex digest as evidence before or when approving.

## KYB registry writer

1. Admin approves a **business** KYC form.
2. Platform provisions a Stellar G-address for the business if missing.
3. Outbox `KYB_REGISTRY_WRITE` invokes `write_attestation(admin, subject, tier, expires_at)`.
4. Deal create, when the KYB gate is on, requires both business counterparties to have a non-revoked, unexpired row.

`subject` is the business G-address, never a user UUID.

## Insurance record writer

1. Back-office insurance review **approved** → `record_policy(deal_id_hash, escrow_id_hash, policy_hash, status)`.
2. Claim decision → `record_claim_decision`.
3. Payout receipt → `record_payout_receipt`.

Hashes must be SHA-256 of the **document bytes** (or a documented canonical encoding) so a partner holding the PDF can match the chain. Mapping S3 keys instead of file bytes will not survive an independent hash check.

`deal_id` / `claim_id` on chain are `BytesN<32>` digests of the off-chain UUIDs, not the UUID ASCII.

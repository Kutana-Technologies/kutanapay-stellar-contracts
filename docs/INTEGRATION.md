# Integration Guide

This guide is for developers who write records to the contracts, or read records from them in an application.

## Before you start

You need:

- the contract ID of each contract you use;
- to write records, the admin account's signing setup;
- a [Stellar SDK](https://developers.stellar.org/docs/tools/sdks) for your language, or the Stellar CLI.

## Writing records

### 1. Derive identifiers

The Insurance Record's `deal_id`, `claim_id` and `escrow_id` are opaque 32-byte values. Use a SHA-256 digest of a stable, canonical identifier from your own system:

```bash
printf '%s' 'your-canonical-id' | sha256sum
```

Recommendations:

- Fix one canonical encoding, such as lowercase hyphenated UUID strings in UTF-8, and document it.
- Never use personal data as an identifier, or as a hash preimage that someone could guess.
- For document hashes, hash the exact file bytes, so that anyone holding the file can recompute the hash and compare.

### 2. Submit the write

Each write takes the admin address as its first argument and must be authorized by that address. With the CLI:

```bash
stellar contract invoke --id <KYB_ID> --network testnet --source-account admin \
  -- write_attestation --admin <ADMIN_ADDRESS> --subject <G-ADDRESS> \
     --tier standard --expires_at 1924992000
```

With an SDK:

1. Build the `invokeHostFunction` transaction.
2. Simulate it, or call `prepareTransaction`.
3. Sign it. If the admin is not the transaction source, also sign the admin's authorization entries.
4. Submit it.

### 3. Handle the result

| Result | Meaning | Action |
|---|---|---|
| Success | The record was written, or an identical record already existed | Done |
| `Error(Contract, #n)` | The request is not valid in the current state (see the [error tables](INTERFACES.md)) | Do not retry the same request. Fix the request or the order of operations. |
| `Error(Auth, …)` | Authorization is missing or wrong | Check the signer configuration |
| Network or timeout error | The outcome is unknown | Retry. Identical writes are idempotent. |

Order-dependent cases:

- **KYB re-approval after revocation:** call `reinstate_attestation`. `write_attestation` fails with `AttestationRevoked`.
- **Payouts:** record the claim decision first. A payout submitted earlier fails with `ClaimNotFound`, and succeeds once the decision is recorded.

## Reading records

### KYB standing

```bash
stellar contract invoke --id <KYB_ID> --network testnet --source-account <any> --send no \
  -- is_valid --subject <G-ADDRESS>
```

Use `is_valid` rather than reimplementing the rule. `get_attestation` returns the raw record, including expired and revoked attestations.

### Insurance records

```bash
stellar contract invoke --id <INS_ID> --network testnet --source-account <any> --send no \
  -- get_claim --claim_id <64-hex>
```

Compare `policy_hash`, `decision_hash` and `receipt_hash` with the SHA-256 of the documents you hold.

### Reading through RPC

Read records through a **simulated contract call**, as shown above, rather than raw ledger-entry lookups. Entries whose TTL has lapsed are archived, and some RPC methods, such as `getLedgerEntries`, do not return archived entries. A simulated call either restores the entry or reports it correctly.

## Indexing events

Every state change emits an event; see the [event reference](INTERFACES.md#events). To follow a contract:

1. Poll RPC `getEvents` with a contract filter.
2. Store the cursor.
3. Process events in ledger order.

RPC providers keep events for a limited window, so persist them yourself if you need the full history.

## Maintaining storage

Writes extend the records they touch. Extend rarely written records periodically; anyone can do this. See [Deployment and operations](DEPLOYMENT.md#maintain-storage-lifetime).

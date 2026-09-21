# Independently verify on-chain state

You do not need KutanaPay software. Use Stellar CLI, Horizon, or [Stellar Expert](https://stellar.expert).

## Escrow (Trustless Work)

Given a contract id `C…` on testnet:

```bash
stellar contract invoke --network testnet --id CCNCAGX7SHZTXRMZQ4RAY3BH34NVC2GOMAN3KI3HXJS7FBWUYMMERBPL -- get_escrow
stellar tx fetch --network testnet --hash <TX_HASH>
```

Expert: `https://stellar.expert/explorer/testnet/contract/<C_ID>` and `/tx/<HASH>`.

Confirm `engagement_id` matches the Trade Secure deal reference (for example `TDS-32077145`) and milestone flags (`approved` / `released`).

## SEP-53 delivery attestation

The public log is the platform `GET /onchain/deals/{id}/attestations` row: `message_hash`, `signature`, `attester_public_key`, canonical JSON.

1. Canonicalize the JSON (sorted keys, no extra whitespace).
2. Verify with SEP-53 (`Stellar Signed Message:\n` + SHA-256, Ed25519) against the attester G-address.
3. The matching on-chain action is the `approve_milestone` transaction for that milestone index, not a separate attestation contract.

The attester used on KutanaPay testnet is `GBRTFYSW76RKCO333MXEH5IPCDXF4UDSR5M5UUXCTMJDEXHCFPNXBUEL`.

## KYB standing

```bash
stellar contract invoke \
  --network testnet \
  --id CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG \
  -- get_attestation --subject G...
```

Treat as valid for trading only if the result is present, `revoked=false`, and `expires_at` is still in the future (unix seconds).

## Insurance record

```bash
stellar contract invoke \
  --network testnet \
  --id CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS \
  -- get_policy --deal_id <32-byte-hash-hex>
```

Hash the policy document with SHA-256 and compare to `policy_hash`. Repeat for claim `decision_hash` and `receipt_hash`.

## SEP-1

Organization accounts: `https://kutanapay.com/.well-known/stellar.toml`

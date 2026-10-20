# Verify On-Chain State

Every claim made by these contracts can be checked independently, using public tools only. You do not need KutanaPay software or credentials.

**Tools:** [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) 28.0.0 and `sha256sum`. Most checks can also be done in a browser with [Stellar Expert](https://stellar.expert/explorer/testnet).

**Setup.** `invoke` needs a source account, even for read-only calls. Any funded testnet account works; it never signs a submitted transaction when `--send no` is used.

```bash
stellar keys generate reader --network testnet --fund

KYB=CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG
INS=CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS
```

## 1. Confirm the deployed code matches this repository

Download the deployed WASM and hash it:

```bash
stellar contract fetch --network testnet --id $KYB -o kyb.wasm && sha256sum kyb.wasm
stellar contract fetch --network testnet --id $INS -o ins.wasm && sha256sum ins.wasm
```

Rebuild from source and compare:

```bash
git clone https://github.com/Kutana-Technologies/kutanapay-stellar-contracts
cd kutanapay-stellar-contracts
git checkout 768f48a          # the commit deployed as v0.2.0
stellar contract build --locked
sha256sum target/wasm32v1-none/release/*.wasm
```

The hashes must be identical. The expected values are listed in [TESTNET.md](TESTNET.md).

> [!NOTE]
> Commit `768f48a` predates the toolchain pin update, so build it with `RUSTUP_TOOLCHAIN=1.98.1`. Later commits pin this version in `rust-toolchain.toml`.

The embedded build metadata shows which toolchain was used:

```bash
stellar contract info meta --network testnet --contract-id $KYB
```

## 2. Confirm who controls the contracts

```bash
stellar contract invoke --network testnet --source-account reader --send no \
  --id $KYB -- get_admin
```

Only this address can write records or upgrade the contract. A change in the contract's WASM hash means it was upgraded. Repeat step 1 for the new code.

## 3. Check a business's KYB standing

```bash
stellar contract invoke --network testnet --source-account reader --send no \
  --id $KYB -- get_attestation --subject <G-ADDRESS>
```

The business is in good standing only if **all** of these hold:

- the result is not `null`
- `revoked` is `false`
- `expires_at` (Unix seconds) is later than now, which you can check with `date +%s`

## 4. Check an insurance record

Derive the identifiers as described in [Identifier and hash encoding](INTEGRATION.md#identifier-and-hash-encoding):

```bash
DEAL_ID=$(printf '%s' '<deal-uuid>' | sha256sum | cut -d' ' -f1)
CLAIM_ID=$(printf '%s' '<claim-uuid>' | sha256sum | cut -d' ' -f1)

stellar contract invoke --network testnet --source-account reader --send no \
  --id $INS -- get_policy --deal_id $DEAL_ID

stellar contract invoke --network testnet --source-account reader --send no \
  --id $INS -- get_claim --claim_id $CLAIM_ID
```

Then confirm the documents you hold:

| Field | Compare with |
|---|---|
| `policy_hash` | `sha256sum policy.pdf` |
| `decision_hash` | `sha256sum claim-decision.pdf`. An all-zero value means no decision was recorded on-chain. |
| `receipt_hash` | `sha256sum payout-receipt.pdf` |

To see when each record was written, and by whom, look up the contract's transaction history on Stellar Expert.

## 5. Verify escrow state (Trustless Work)

```bash
stellar contract invoke --network testnet --source-account reader --send no \
  --id CCNCAGX7SHZTXRMZQ4RAY3BH34NVC2GOMAN3KI3HXJS7FBWUYMMERBPL -- get_escrow
```

Confirm that `engagement_id` matches the Trade Secure deal reference (for example `TDS-32077145`), and check each milestone's `approved` and `released` flags.

## 6. Verify a delivery attestation (SEP-53)

KutanaPay publishes each delivery attestation with its canonical JSON message, `message_hash`, `signature` and `attester_public_key`. To verify one:

1. Canonicalize the JSON: keys sorted, no insignificant whitespace.
2. Compute the SEP-53 message hash: `SHA-256("Stellar Signed Message:\n" + message)`.
3. Verify the Ed25519 signature against the attester address listed in [TESTNET.md](TESTNET.md).
4. Find the matching `approve_milestone` transaction for that milestone on the escrow contract.

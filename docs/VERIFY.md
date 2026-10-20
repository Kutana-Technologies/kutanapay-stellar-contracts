# Verification Guide

You can verify everything these contracts publish yourself, using only public tools.

**You need:** [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) 28.0.0 and `sha256sum`. [Stellar Expert](https://stellar.expert) offers a browser alternative for most steps.

**Setup:** `invoke` needs a source account even for read-only calls. Any funded account works. With `--send no`, nothing is submitted.

```bash
stellar keys generate reader --network testnet --fund
ID=<CONTRACT_ID>
```

## 1. Verify the deployed code

1. Download the deployed WASM and hash it:

   ```bash
   stellar contract fetch --network testnet --id $ID -o deployed.wasm
   sha256sum deployed.wasm
   ```

2. Find out which version is deployed:

   ```bash
   stellar contract info meta --network testnet --contract-id $ID   # binver, rsver, cliver
   ```

3. Rebuild that version from source:

   ```bash
   git clone https://github.com/Kutana-Technologies/kutanapay-stellar-contracts
   cd kutanapay-stellar-contracts
   git checkout <commit for that version, see CHANGELOG.md>
   stellar contract build --locked
   sha256sum target/wasm32v1-none/release/*.wasm
   ```

4. Compare the hashes. They must be identical.

> [!NOTE]
> Build with the Rust version shown in `rsver`. Releases from 0.2.1 onward pin it in `rust-toolchain.toml`. For 0.2.0, set `RUSTUP_TOOLCHAIN=1.98.1`.

## 2. Verify who controls the contract

```bash
stellar contract invoke --network testnet --source-account reader --send no --id $ID -- get_admin
stellar contract invoke --network testnet --source-account reader --send no --id $ID -- get_pending_admin
```

To review past upgrades and admin changes, look up the contract's `upgraded`, `adm_prop`, `adm_set` and `adm_cncl` events.

## 3. Verify KYB standing

```bash
stellar contract invoke --network testnet --source-account reader --send no \
  --id $ID -- is_valid --subject <G-ADDRESS>
```

To see the underlying record (tier, expiry, revocation), call `get_attestation` with the same argument.

## 4. Verify an insurance record

1. Compute the identifiers, using the encoding documented by the record's publisher:

   ```bash
   CLAIM_ID=$(printf '%s' '<claim-identifier>' | sha256sum | cut -d' ' -f1)
   ```

2. Read the records:

   ```bash
   stellar contract invoke --network testnet --source-account reader --send no \
     --id $ID -- get_policy --deal_id <DEAL_ID>
   stellar contract invoke --network testnet --source-account reader --send no \
     --id $ID -- get_claim --claim_id $CLAIM_ID
   ```

3. Compare each hash with the SHA-256 of the corresponding document you hold:

   | Field | Document |
   |---|---|
   | `policy_hash` | Policy |
   | `decision_hash` | Claim decision |
   | `receipt_hash` | Payout receipt |

To find when each record was written, and by which transaction, use the contract's `policy`, `decision` and `payout` events or its transaction history on Stellar Expert.

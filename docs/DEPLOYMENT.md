# Deployment and Operations

This guide covers the full operational lifecycle of a contract instance: deploying it, verifying it, upgrading it, transferring the admin role, and keeping storage alive.

Examples use testnet. Replace `--network testnet` for other networks.

## Choose the admin

The admin is the only address that can write records or upgrade the contract.

- Use a **dedicated** account that has no other role.
- Use a **[multisig account](https://developers.stellar.org/docs/learn/encyclopedia/security/signatures-multisig)** with a threshold of two or more independent signers, held on hardware or KMS-backed keys. Contract authorization enforces the account's thresholds automatically.
- Never store signing keys in source control or CI.

## Deploy

1. Build, and record the hashes:

   ```bash
   stellar contract build --locked
   sha256sum target/wasm32v1-none/release/*.wasm
   ```

2. Deploy each contract with the admin address:

   ```bash
   stellar contract deploy \
     --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
     --network testnet --source-account deployer \
     -- --admin <ADMIN_ADDRESS>

   stellar contract deploy \
     --wasm target/wasm32v1-none/release/insurance_record.wasm \
     --network testnet --source-account deployer \
     -- --admin <ADMIN_ADDRESS>
   ```

3. Verify the deployment (see [VERIFY.md](VERIFY.md)):

   ```bash
   stellar contract fetch --network testnet --id <CONTRACT_ID> -o deployed.wasm && sha256sum deployed.wasm
   stellar contract invoke --network testnet --source-account deployer --send no \
     --id <CONTRACT_ID> -- get_admin
   ```

## Upgrade in place

An upgrade replaces the code but keeps the contract ID and all storage.

1. Build the new release and confirm its hashes match the release notes.
2. Upload the WASM and note the returned hash:

   ```bash
   stellar contract upload --wasm target/wasm32v1-none/release/kyb_attestation_registry.wasm \
     --network testnet --source-account admin
   ```

3. Invoke `upgrade` as the admin:

   ```bash
   stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account admin \
     -- upgrade --new_wasm_hash <WASM_HASH>
   ```

4. Verify: `binver` in `stellar contract info meta`, the fetched WASM hash, and the `upgraded` event.

### Upgrading from 0.2.0

Version 0.3.0 keeps the 0.2.0 function signatures and storage layout, so existing records remain readable after the upgrade. The `upgrade-tests` crate exercises this path. Notes:

- The `upgrade` call runs the 0.2.0 code, so this upgrade does not emit an `upgraded` event. Later upgrades do.
- Claims written by 0.2.0 that have `decision = "paid"` and an all-zero `decision_hash` hold a receipt but no decision. Record their decision with `record_claim_decision`; the receipt is kept.
- Writers must follow the 0.3.0 rules described in the [integration guide](INTEGRATION.md#3-handle-the-result).

## Transfer the admin role

1. The current admin nominates the new address:

   ```bash
   stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account admin \
     -- propose_admin --new_admin <NEW_ADMIN>
   ```

2. The nominee accepts:

   ```bash
   stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account new-admin \
     -- accept_admin
   ```

3. Confirm the change with `get_admin`, and update every writer's configuration.

To withdraw a nomination before it is accepted, the current admin calls `cancel_admin_transfer`. Run the procedure for each contract instance.

## Maintain storage lifetime

Every write extends the written record to about 120 days and the contract instance to about 30 days. To keep rarely written data alive, extend it on a schedule. These calls need no authorization; the caller pays the fee.

| What | Call |
|---|---|
| Contract instance and code | `extend_ttl` |
| One KYB attestation | `extend_attestation_ttl --subject <G-ADDRESS>` |
| One policy / claim | `extend_policy_ttl --deal_id <hex>` / `extend_claim_ttl --claim_id <hex>` |

For example:

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet --source-account <payer> -- extend_ttl
```

If an entry has already been archived, the next invocation that touches it restores it automatically (Protocol 23 and later), at extra fee.

## Monitoring

Subscribe to the contract's events (see [INTEGRATION.md](INTEGRATION.md#indexing-events)) and alert on:

- `upgraded`: any code change;
- `adm_prop`, `adm_set` and `adm_cncl`: any change to the admin role.

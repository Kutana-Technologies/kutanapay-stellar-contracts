# Testnet Deployments

**Network:** Stellar testnet (`Test SDF Network ; September 2015`)
**Last verified:** 20 October 2026

These are the Trade Secure review deployments. They may be redeployed. Always confirm current values on-chain using the commands in [VERIFY.md](VERIFY.md).

## Contracts

| Contract | Contract ID | Version | WASM SHA-256 |
|---|---|---|---|
| KYB Attestation Registry | `CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG` | 0.2.0 | `6c06e0ecafe9616903f0ceef2a8ada208aafb26404ace1deb39ee89720218747` |
| Insurance Record | `CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS` | 0.2.0 | `ffeae58c889f2835cc9716e81724c229703584c26446f18862979a9669563452` |

Both WASM hashes are reproducible from commit [`768f48a`](https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/tree/768f48a). Build it with Rust 1.98.1 and Stellar CLI 28.0.0; see [VERIFY.md](VERIFY.md#1-confirm-the-deployed-code-matches-this-repository).

Explorer links:

- [KYB Attestation Registry on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG)
- [Insurance Record on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS)

## Public role addresses

These are public account addresses, not secrets.

| Role | Address |
|---|---|
| Contract admin (both contracts; check with `get_admin`) | `GBRTFYSW76RKCO333MXEH5IPCDXF4UDSR5M5UUXCTMJDEXHCFPNXBUEL` |
| Delivery attester (SEP-53) | `GBRTFYSW76RKCO333MXEH5IPCDXF4UDSR5M5UUXCTMJDEXHCFPNXBUEL` |
| Escrow approver | `GCR6RMBXZQWXOZ3AXVFKJ4YVYAU5HHSY3OXMPS3RQCWPYW6VCAQXZRYC` |
| Escrow release signer | `GDZSCG56SIV5BQ5QTRMUCURAUSULXUD2UPRZ7LPMAA2AK4ECF3RUDZJT` |
| Escrow dispute resolver | `GAR2EMYN6UX4PDPHEREX6DXNJGHSORWYLD53K7B4EN2OA7KGX6GP2AN3` |
| Platform, treasury, receiver | `GBDMBGDZ6FZRESX34XE6XNOKWYAXFNL6H756IMAO7US3BLRTHUKX22LK` |

## Example escrow

A Trustless Work escrow for Trade Secure deal `TDS-32077145`: `CCNCAGX7SHZTXRMZQ4RAY3BH34NVC2GOMAN3KI3HXJS7FBWUYMMERBPL`.

## Organization accounts

KutanaPay publishes its Stellar organization accounts in its SEP-1 file: <https://kutanapay.com/.well-known/stellar.toml>.

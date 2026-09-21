# Testnet deployments

Network: **Test SDF Network ; September 2015** (`stellar network testnet`).

These ids are the KutanaPay Trade Secure review deployment as of 21 Sep 2026. They can change if the contracts are redeployed. `stellar contract info interface --network testnet --id …` is the source of truth.

| Contract | Id |
|----------|----|
| KYB Attestation Registry | `CDCI7A4S4PK2NBTS5R3TIGLWZX5IDARBU6JC75I5TFGQBOLEUGY2BEOG` |
| Insurance Record | `CA72PPLD6EBTSETH4HPVK35QK6DAUSK4TYKJENK2ZEG4XNRM5V2LU3DS` |

Example Trustless Work escrow for deal **TDS-32077145**: `CCNCAGX7SHZTXRMZQ4RAY3BH34NVC2GOMAN3KI3HXJS7FBWUYMMERBPL`.

Public roles used by that review stack (not secrets):

| Role | Address |
|------|---------|
| Attester (SEP-53) | `GBRTFYSW76RKCO333MXEH5IPCDXF4UDSR5M5UUXCTMJDEXHCFPNXBUEL` |
| Approver | `GCR6RMBXZQWXOZ3AXVFKJ4YVYAU5HHSY3OXMPS3RQCWPYW6VCAQXZRYC` |
| Release signer | `GDZSCG56SIV5BQ5QTRMUCURAUSULXUD2UPRZ7LPMAA2AK4ECF3RUDZJT` |
| Dispute resolver | `GAR2EMYN6UX4PDPHEREX6DXNJGHSORWYLD53K7B4EN2OA7KGX6GP2AN3` |
| Platform / treasury / receiver / deploy | `GBDMBGDZ6FZRESX34XE6XNOKWYAXFNL6H756IMAO7US3BLRTHUKX22LK` |

Do not put signing secrets in this repository.

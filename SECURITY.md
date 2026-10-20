# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities **privately**. Do not open a public issue, pull request, or discussion.

Use GitHub private vulnerability reporting: open the repository's **Security** tab and choose **Report a vulnerability**, or go to <https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/security/advisories/new>.

Please include:

- the affected contract and version (or commit)
- a description of the issue and its impact
- steps to reproduce, ideally as a failing `cargo test` case
- any suggested fix

## What to expect

| Stage | Target |
|---|---|
| Acknowledgement | within 3 business days |
| Initial assessment | within 10 business days |
| Fix and coordinated disclosure | agreed with the reporter, based on severity |

We will credit reporters in the advisory unless they prefer to remain anonymous.

## Scope

| In scope | Out of scope |
|---|---|
| Contract code in this repository | Trustless Work escrow contracts (report to Trustless Work) |
| Build and release configuration in this repository | The Stellar network, Soroban host, or Stellar CLI (report to the [Stellar Development Foundation](https://github.com/stellar)) |
| Contract instances listed in [docs/TESTNET.md](docs/TESTNET.md) | KutanaPay web or mobile applications |

## Supported versions

| Version | Supported |
|---|---|
| 0.2.x | Yes (testnet only) |
| earlier than 0.2 | No |

For trust assumptions and known limitations, see the [security model](docs/SECURITY.md).

# Security Policy

## Reporting a vulnerability

Report suspected vulnerabilities **privately**. Do not open a public issue, pull request or discussion about them.

To report one, open the repository's **Security** tab and choose **Report a vulnerability**, or go to <https://github.com/Kutana-Technologies/kutanapay-stellar-contracts/security/advisories/new>.

Please include:

- the affected contract and version or commit;
- a description of the issue and its impact;
- steps to reproduce, ideally a failing `cargo test` case;
- a suggested fix, if you have one.

## What to expect

| Stage | Target |
|---|---|
| Acknowledgement | Within 3 business days |
| Initial assessment | Within 10 business days |
| Fix and coordinated disclosure | Agreed with the reporter, according to severity |

Reporters are credited in the published advisory unless they ask to remain anonymous.

## Scope

| In scope | Out of scope |
|---|---|
| Contract source in this repository | The Stellar network, the Soroban host and the Stellar CLI (report these to the [Stellar Development Foundation](https://github.com/stellar)) |
| Build and release configuration in this repository | Third-party contracts and applications |
| Deployments listed in [docs/TESTNET.md](docs/TESTNET.md) | |

## Supported versions

| Version | Supported |
|---|---|
| 0.3.x | Yes |
| 0.2.x | Until the reference deployments are upgraded |

For the trust assumptions and the guarantees the contracts enforce, see the [security model](docs/SECURITY.md).

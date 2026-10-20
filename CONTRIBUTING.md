# Contributing

Thank you for your interest in improving these contracts.

## Development setup

1. Install [rustup](https://rustup.rs/) and [Stellar CLI](https://developers.stellar.org/docs/tools/cli/install-cli) 28.0.0.
2. Clone the repository. rustup installs the pinned toolchain automatically the first time you run `cargo`.
3. Confirm that everything passes:

   ```bash
   stellar contract build --locked
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   ```

## Making changes

- **Keep storage compatible.** Do not change existing storage keys or record layouts unless an upgrade path is tested in `upgrade-tests`.
- **Keep the interface stable.** Changing a function signature breaks existing callers. Add new functions instead.
- **Test every rule.** Each new check needs a test that fails when the check is removed.
- **Emit events** for new state changes, and document them in [docs/INTERFACES.md](docs/INTERFACES.md).
- **Use typed errors.** Add new error codes at the end of the enum; never renumber existing codes.
- **Update the documentation** and the [changelog](CHANGELOG.md) in the same pull request.

## Pull requests

1. Create a branch from `main`.
2. Keep each pull request focused on one change, and describe its motivation and testing.
3. CI must pass before review.

## Reporting security issues

Do not open public issues for vulnerabilities. See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contributions are licensed under the [Apache License 2.0](LICENSE).

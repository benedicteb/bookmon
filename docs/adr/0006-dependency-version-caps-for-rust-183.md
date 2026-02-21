# 0006 - Dependency version caps for Rust 1.83 compatibility

## Status

Accepted

## Context

Several crate ecosystems have adopted Rust edition 2024 and/or MSRV 1.85+, including:
- `getrandom` 0.4 (used by `uuid` 1.21+ and `tempfile` 3.25+)
- `security-framework` 3.x (used by `native-tls` 0.2.18+)

Our toolchain is Rust 1.83, which cannot even parse manifests using `edition2024`. This means any transitive dependency using edition 2024 causes a hard build failure at the download stage.

## Decision

Cap the following dependency versions in Cargo.toml to maintain Rust 1.83 compatibility:

- `uuid`: `>=1.17, <1.21` (1.21 introduces getrandom 0.4 dependency)
- `tempfile`: `>=3.20, <3.25` (3.25 introduces getrandom 0.4 dependency)
- Pin `native-tls` to 0.2.14 in Cargo.lock (0.2.18 requires security-framework 3.x)

Additionally, `reqwest` stays at 0.12.x (not 0.13) since 0.13 is a separate semver-major upgrade with TLS backend changes.

## Consequences

- These caps must be revisited and removed once the Rust toolchain is upgraded to 1.85+.
- The lock file pin on `native-tls` is fragile; a `cargo update` will respect it, but regenerating the lock file from scratch may re-resolve to the incompatible version. A manual `cargo update -p native-tls@<new> --precise 0.2.14` step may be needed.
- We cannot benefit from improvements in uuid 1.21+, tempfile 3.25+, or native-tls 0.2.18+ until the toolchain upgrade.

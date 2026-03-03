# CI and Release

This document describes the GitHub Actions workflows shipped with `os_service_fabric`.

## CI workflow (`.github/workflows/ci.yml`)

The CI workflow runs on:

- `push` to `main`
- `pull_request`

It enforces a strict, warning-free standard:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo doc` with `RUSTDOCFLAGS="-D warnings"`

It also performs a `no_std` compilation check by building for `x86_64-unknown-none`
without default features.

## Release workflow (`.github/workflows/release.yml`)

The release workflow runs on:

- `push` of a tag matching `v*` (e.g. `v0.1.0`)
- manual trigger (`workflow_dispatch`)

It publishes to crates.io using:

- `cargo publish --token "$CARGO_REGISTRY_TOKEN"`

### Required secret

Set the following GitHub Actions secret in your repository settings:

- `CARGO_REGISTRY_TOKEN`: a crates.io token with publish permission

## Recommended release process

1. Ensure `Cargo.toml` version is updated.
2. Ensure CI is green on `main`.
3. Create and push a tag:
   - `v0.1.0`
4. The release workflow publishes the crate.


# Changelog

All notable changes to this project will be documented in this file.

This project follows a lightweight variant of Keep a Changelog.

## [0.1.0] - 2026-03-03

### Added

- Dependency-free `no_std` service lifecycle trait (`Service`)
- Static, allocation-free dependency graph (`ServiceGraph`, `ServiceNode`)
- Deterministic topological sorting and validation
- Recovery policies (`RecoveryPolicy`, `RecoveryAction`)
- Fabric runtime (`FabricRuntime`) with boot, step, and shutdown
- Strict CI and automated crates.io publishing workflows


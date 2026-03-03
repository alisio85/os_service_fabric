# Testing Strategy

This document outlines a pragmatic testing approach for `os_service_fabric` users.

## 1. Unit test each service in isolation

You can test a service by providing a simple host-side context and calling:

- `init`
- `start`
- `health`
- `stop`

Because `ServiceError` and `ServiceHealth` are allocation-free, these tests can run
even in constrained environments.

## 2. Graph-level tests

Use `ServiceGraph` + `FabricRuntime` to test:

- correct boot order (dependencies before dependents)
- missing dependency detection
- cycle detection
- shutdown order (reverse topological)

## 3. Health and recovery tests

Create a service that reports:

- `Ok` initially
- then `Failed`

and verify the runtime applies your `RecoveryPolicy`.

## 4. no_std compilation checks


# Recovery Policies

This document describes how `os_service_fabric` handles failures using recovery policies.

## Actions

`RecoveryAction` can be:

- `Retry { max_retries }`: attempt the operation again a limited number of times.
- `Restart`: best-effort stop, then re-init and re-start (for init failures) or mark `Created` (for runtime failures).
- `MarkDegraded`: keep running, but mark the lifecycle as `Degraded`.
- `Fail`: fail fast.

## Policy design philosophy

Kernel subsystems vary widely in criticality and recoverability:

- memory management failures are typically fatal
- a debug shell failure should not prevent boot
- a network stack may be restartable

Policies let you encode those decisions explicitly and keep the runtime generic.

## Initialization failures

Initialization failures are errors returned from `Service::init` or `Service::start` during `FabricRuntime::boot`.

Typical recommendations:

- critical services (memory, scheduler): `Fail`
- optional services (debug console): `MarkDegraded`

### Retry semantics

`Retry { max_retries }` means:

- total attempts = `1 + max_retries`
- each attempt calls `init` then `start`
- after a failed attempt, the runtime calls `stop` best-effort

This encourages services to make `stop` safe even after partial init.

### Restart semantics

`Restart` during initialization is treated as:

- best-effort `stop`
- then `init + start` again once

## Runtime failures

Runtime failures are health checks that report `HealthLevel::Failed`.

Typical recommendations:

- crash-only systems: `Fail`
- robust systems: `Restart` for non-critical services, `Fail` for critical services

### Restart on runtime failure

For runtime failures, `Restart` marks the service as `Created` after calling `stop`.
This keeps `step()` small and deterministic and allows the embedding kernel to decide:

- call `boot()` again (full re-orchestration)
- implement a custom per-service restart loop
- escalate to panic if the failure is too frequent

## Example policy table

- **memory**: init = Fail, runtime = Fail
- **scheduler**: init = Fail, runtime = Fail
- **device-framework**: init = Fail, runtime = Restart
- **network**: init = MarkDegraded, runtime = Restart
- **debug-console**: init = MarkDegraded, runtime = MarkDegraded



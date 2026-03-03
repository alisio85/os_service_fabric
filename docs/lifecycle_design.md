# Lifecycle Design

This document explains the lifecycle model used by `os_service_fabric`.

## Lifecycle states

The fabric models each service as being in one of the following states:

- `Created`
- `Initializing`
- `Running`
- `Degraded`
- `Stopping`
- `Stopped`
- `Failed`

The fabric runtime is the primary driver of these transitions.

## State vs health

Lifecycle state is a **control-plane** concept: it indicates what the fabric
believes should be happening.

Health level is a **data-plane** concept: it reflects what the service reports
about its current operating condition.

It is normal for a service to be in `Running` while reporting degraded health.

## Recommended service behavior

- Make `init` and `start` idempotent where possible.
- Make `stop` best-effort and safe to call after partial initialization.
- Keep `health` cheap and allocation-free.


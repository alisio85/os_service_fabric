# Dependency Graph

This document describes how `os_service_fabric` models service dependencies.

## Declaring dependencies

Each `ServiceNode` declares its dependencies as a static slice of `ServiceId`:

- `B` depends on `A` means: `A` must be initialized and started before `B`.

Dependencies should represent **hard requirements**. If your service can run without
another subsystem (e.g. a debug shell without a filesystem), prefer not declaring a
hard dependency. Instead:

- keep the service independent
- check for optional capability availability at runtime in your `Context`
- report degraded health if the optional capability is missing

## Validation

`ServiceGraph::validate()` checks:

- every dependency is present in the graph
- the graph contains no cycles
- the node's declared id matches the service-reported id

### Missing dependency detection

The validator checks every `dep` in every `ServiceNode` against the set of node IDs.
If a dependency is missing, validation fails with `GraphError::MissingDependency(dep)`.

This catches wiring errors early (ideally in CI).

### Cycle detection

Cycles can occur accidentally when two subsystems depend on each other, e.g.:

- `fs` depends on `vfs-cache`
- `vfs-cache` depends on `fs`

Or via a longer cycle. The validator detects cycles by attempting a topological sort.
If not all nodes can be sorted, the graph contains a cycle.

## Topological ordering

`ServiceGraph::topo_order()` returns indices such that dependencies appear before
dependents (classic DAG topological sorting).

The implementation is allocation-free and deterministic, intended for
kernel and CI usage.

### Determinism

When multiple nodes are eligible at once (indegree 0), the reference implementation
chooses them in ascending index order. This means:

- the output is stable across runs
- tests can assert exact ordering

If you want a different priority scheme, you can reorder your `nodes` slice.

## Limits

The reference implementation supports up to `MAX_SERVICES` nodes (default: 64).
This keeps internal scratch arrays fixed-size.

If you need more, you can increase `MAX_SERVICES` and rebuild the crate.



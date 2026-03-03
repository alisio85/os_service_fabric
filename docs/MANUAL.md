# os_service_fabric Manual

## 1. Introduction

`os_service_fabric` is a dependency-free, `no_std`-first framework for modeling and
orchestrating operating system services in Rust (edition 2024).

It focuses on:

- explicit service lifecycles
- static dependency graphs (DAG)
- health reporting and recovery policies
- deterministic, allocation-free runtimes

This manual explains how to use the crate step by step, from basic concepts to
integration patterns that fit real kernels.

## 2. Core concepts

### 2.1 Services

A **service** is any OS subsystem that can be driven by a lifecycle:

- `init`: prepare internal state
- `start`: become operational
- `stop`: gracefully stop
- `health`: report a coarse-grained health snapshot

Typical services:

- physical memory manager
- virtual memory manager
- scheduler
- filesystem
- network stack
- device framework
- IPC layer

### 2.2 Service identifiers (`ServiceId`)

The fabric uses [`ServiceId`](../src/id.rs) as a stable, human-readable identifier.
It is typically created from a `&'static str`, which is:

- allocation-free
- stable over time
- easy to use in logs and diagnostics

### 2.3 Lifecycle state vs health

`os_service_fabric` intentionally separates:

- **Lifecycle state**: what the fabric believes should be happening (`Created`, `Running`, `Stopped`, …).
- **Health level**: what the service reports about its condition (`Ok`, `Degraded`, `Failed`).

This separation is useful for OS kernels because "partially working" subsystems are common:

- your network stack might be running but without DHCP (`Degraded`)
- your entropy source might be weak early in boot (`Degraded`)
- your filesystem may have remounted read-only after an error (`Degraded`)

The fabric lets you keep the OS alive and explicit about health.

## 3. Getting started

### 3.1 Add the dependency

In your kernel or OS workspace, add:

```toml
[dependencies]
os_service_fabric = "0.1"
```

If you are using it in `no_std`, you typically do not enable any features.

### 3.2 no_std and std feature

The crate is `#![no_std]`. Unit tests and doc tests run on the host using `std`
automatically (Rust's test harness uses `std`).

The optional `std` feature exists only as a compatibility hook for users who want
to gate additional convenience code in their own projects. The core crate does not
allocate regardless of `std`.

## 4. Writing a service

### 4.1 The `Service` trait

To integrate a subsystem, implement the `Service` trait:

```rust
use os_service_fabric::{Service, ServiceError, ServiceHealth, ServiceId};

pub struct MemoryService;

impl Service for MemoryService {
    type Context = ();

    fn id(&self) -> ServiceId {
        ServiceId::new("memory")
    }

    fn init(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Initialize allocators, page tables, etc.
        Ok(())
    }

    fn start(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Start accepting allocations.
        Ok(())
    }

    fn stop(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Best-effort cleanup.
        Ok(())
    }

    fn health(&self) -> ServiceHealth {
        ServiceHealth::ok()
    }
}
```

### 4.2 The `Context` pattern

The `Context` associated type is your bridge from "fabric generic code" to your kernel.

Common patterns:

- `type Context = ();`
  - simplest for small kernels that store global state elsewhere
- `type Context = KernelContext;`
  - a single struct containing handles/capabilities used by all services
- `type Context = BootContext;`
  - a boot-only context for init/start; later runtime can switch to another phase

Design advice:

- Prefer passing *capabilities* rather than "god objects".
- Keep access explicit: `ctx.serial.write(...)`, `ctx.arch.enable_irq(...)`, etc.
- Keep it deterministic for host-side tests (mock the context).

## 5. Declaring dependencies

### 5.1 Service nodes

You declare a service node using `ServiceNode::new(id, deps, service)`, where `deps`
is a static slice of `ServiceId`.

Example:

```rust
use os_service_fabric::{ServiceId, ServiceNode};

static DEPS_FS: &[ServiceId] = &[
    ServiceId::new("memory"),
    ServiceId::new("block"),
];

// filesystem node depends on memory and block device framework
// ServiceNode::new(ServiceId::new("fs"), DEPS_FS, FilesystemService { ... })
```

### 5.2 Build and validate the graph

```rust
use os_service_fabric::{ServiceGraph, GraphError};

fn build_graph<'a, S: os_service_fabric::Service>(
    nodes: &'a mut [os_service_fabric::ServiceNode<S>],
) -> Result<ServiceGraph<'a, S>, GraphError> {
    let graph = ServiceGraph::new(nodes)?;
    graph.validate()?;
    Ok(graph)
}
```

Validation checks:

- missing dependencies
- cycles
- wiring mistakes: `ServiceNode::id` must match `service.id()`

## 6. Recovery policies

Each service has a `RecoveryPolicy` with:

- `action_on_init_failure`
- `action_on_runtime_failure`

Example policy table:

- **Memory**: init failure = `Fail`, runtime failure = `Fail`
- **Filesystem**: init failure = `Fail`, runtime failure = `MarkDegraded`
- **Debug console**: init failure = `MarkDegraded`, runtime failure = `MarkDegraded`

See `docs/recovery_policies.md` for deep details.

## 7. Driving the runtime

### 7.1 Boot

`FabricRuntime::boot(&mut ctx)` performs:

- graph validation
- topological sorting
- `init` then `start` in dependency order
- policy-driven recovery if `init`/`start` fail

### 7.2 Step loop

`FabricRuntime::step(&mut ctx)` is designed to be called:

- periodically (timer tick)
- opportunistically (after important events)
- or during idle loops

It checks health for services in `Running` or `Degraded` state and updates lifecycle
state accordingly.

### 7.3 Shutdown

`FabricRuntime::shutdown(&mut ctx)` performs best-effort stop in *reverse*
topological order (dependents stop before dependencies).

## 8. Determinism and safety

### 8.1 Deterministic ordering

When multiple nodes are available to start (indegree 0), the reference implementation
uses ascending index order to keep the output deterministic.

### 8.2 No allocation

All APIs are allocation-free. Any temporary scratch memory used by the graph and runtime
is stored in fixed-size arrays bounded by `MAX_SERVICES` (default 64).

### 8.3 No I/O

The crate performs no logging or device I/O. If you want logs, do it inside services,
or integrate with your own kernel logging framework.

## 9. Testing

See `docs/testing_strategy.md` for:

- unit tests
- graph-level tests
- recovery behavior tests
- `no_std` compilation checks and CI patterns

## 10. License and attribution

MIT License.

Attribution requirement: this crate explicitly states it was created by an AI assistant
(ChatGPT) based on an idea by **alisio85**.


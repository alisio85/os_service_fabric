# os_service_fabric

`os_service_fabric` is a **dependency-free**, `no_std`-first crate that provides
a small but expressive framework for modeling and orchestrating **operating
system services** in Rust (edition **2024**).

It focuses on:

- explicit, stateful **service lifecycles**
- static, allocation-free **dependency graphs** (DAG)
- **health reporting** and **recovery policies**
- a deterministic, allocation-free **runtime** that drives your services

This crate is designed to complement, but not depend on, OS development crates like:

- `os_kernel_foundry` — kernel and boot orchestration
- `os_dev_toolkit` — developer tooling (logging, diagnostics)
- `os_metal_primitives` — low-level MMIO, bitfields, IRQ, and register primitives

`os_service_fabric` handles a **different concern**: the *service-level fabric*
that ties together your memory manager, scheduler, filesystem, device frameworks,
and other subsystems as first-class services with well-defined lifecycles.

## Design goals

- `no_std`-first with an optional `std` feature
- zero external dependencies
- deterministic, allocation-free core API
- clear, fully documented traits and types
- CI-friendly: no warnings allowed (including docs)

## Quick example

```rust
use os_service_fabric::{
    Service, ServiceError, ServiceHealth, ServiceId, ServiceState,
    ServiceGraph, FabricRuntime, RecoveryAction, RecoveryPolicy,
};

struct SchedulerService;

impl Service for SchedulerService {
    type Context = ();

    fn id(&self) -> ServiceId {
        ServiceId::new("scheduler")
    }

    fn init(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Initialize internal scheduler state here.
        Ok(())
    }

    fn start(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Start accepting tasks.
        Ok(())
    }

    fn stop(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
        // Flush queues, quiesce execution.
        Ok(())
    }

    fn health(&self) -> ServiceHealth {
        ServiceHealth::ok()
    }
}

fn minimal_demo() {
    let scheduler = SchedulerService;

    static DEPS: &[ServiceId] = &[];
    let mut nodes = [
        os_service_fabric::ServiceNode::new(
            ServiceId::new("scheduler"),
            DEPS,
            scheduler,
        ),
    ];

    let graph = ServiceGraph::new(&mut nodes).expect("valid graph");

    let policies = [
        RecoveryPolicy {
            action_on_init_failure: RecoveryAction::Fail,
            action_on_runtime_failure: RecoveryAction::MarkDegraded,
        },
    ];

    let mut states = [ServiceState::Created];

    let mut runtime = FabricRuntime::new(graph, &policies, &mut states)
        .expect("policies and states match graph");

    let mut ctx = ();
    runtime.boot(&mut ctx).expect("boot scheduler");
}
```

For a complete walkthrough, see [`docs/MANUAL.md`](docs/MANUAL.md).

## Documentation

- API docs: built on docs.rs (`cargo doc --open` locally)
- Manual: `docs/MANUAL.md`
- Additional documents:
  - `docs/lifecycle_design.md`
  - `docs/dependency_graph.md`
  - `docs/recovery_policies.md`
  - `docs/integration_patterns.md`
  - `docs/testing_strategy.md`

## License and attribution

Licensed under the MIT License (see `LICENSE`).

This crate was **created by an AI assistant (ChatGPT)** based on an idea and
requirements by **alisio85**.


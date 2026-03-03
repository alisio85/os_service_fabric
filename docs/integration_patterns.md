# Integration Patterns

This document describes common ways to integrate `os_service_fabric` into a kernel.

## Boot integration

1. Construct your services (memory, scheduler, filesystem, etc.).
2. Build `ServiceNode` values with static dependency slices.
3. Create a `ServiceGraph` and a `FabricRuntime`.
4. Call `runtime.boot(&mut ctx)`.

## Using a kernel-wide context

Most kernels already have a central "kernel context" struct that holds:

- architecture hooks
- allocators
- console output
- device access
- configuration

You can pass `&mut KernelContext` as `Service::Context`.

## Health and monitoring

The fabric does not prescribe how you export health status.

Common approaches:

- periodically call `runtime.step(&mut ctx)` and map state transitions into logs
- expose `runtime.states()` through a debug shell command
- report aggregated health to a hypervisor or external monitor

## Relationship with other crates

`os_service_fabric` is independent and does not depend on your other crates:

- `os_kernel_foundry`: boot and architecture traits
- `os_dev_toolkit`: logging and diagnostics
- `os_metal_primitives`: MMIO/bitfields/IRQ primitives

You can wrap those building blocks inside services that implement the fabric traits.


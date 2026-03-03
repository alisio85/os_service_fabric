//! Definition of the `Service` trait implemented by subsystems.
//!
//! A "service" is any OS subsystem that can be driven by a lifecycle:
//!
//! - initialization
//! - start (becoming operational)
//! - stop (graceful shutdown)
//! - periodic or on-demand health checks
//!
//! Typical examples:
//!
//! - physical/virtual memory managers
//! - schedulers
//! - filesystems
//! - device frameworks
//! - networking stacks
//! - IPC frameworks
//!
//! The fabric is intentionally agnostic about architecture, I/O, allocation and
//! concurrency model. This makes it a good fit for early-boot kernels and `no_std`
//! environments.

use crate::{error::ServiceError, health::ServiceHealth, id::ServiceId};

/// A service managed by the `os_service_fabric` runtime.
///
/// # Context
///
/// Each service uses a shared `Context` type (associated type) to receive handles,
/// configuration, or capability access from the embedding kernel. The fabric does
/// not define what this context contains.
///
/// Many kernels will use `()` as the context type and store global state elsewhere;
/// others may pass a mutable reference to a "kernel context" struct.
///
/// # Error model
///
/// Errors are represented as [`ServiceError`], which remains allocation-free.
pub trait Service {
    /// Shared context type passed to service operations.
    type Context;

    /// Returns this service's stable identifier.
    fn id(&self) -> ServiceId;

    /// Prepare the service for running.
    ///
    /// This should ideally be idempotent. The fabric may call `init` multiple times
    /// when recovery policies request retries or restarts.
    fn init(&mut self, ctx: &mut Self::Context) -> Result<(), ServiceError>;

    /// Enter the "running" state.
    ///
    /// This is typically called after all dependencies have successfully reached `Running`.
    fn start(&mut self, ctx: &mut Self::Context) -> Result<(), ServiceError>;

    /// Request a graceful stop.
    ///
    /// Implementations should attempt best-effort cleanup. The fabric may call this
    /// even after partial initialization.
    fn stop(&mut self, ctx: &mut Self::Context) -> Result<(), ServiceError>;

    /// Perform a one-shot health check.
    ///
    /// The health check should not allocate and should ideally be fast and deterministic.
    fn health(&self) -> ServiceHealth;
}

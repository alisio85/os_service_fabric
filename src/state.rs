//! Service lifecycle states and related helpers.

/// Describes the lifecycle state of a service within the fabric.
///
/// The fabric runtime performs state transitions in a controlled way.
/// While the runtime is the primary driver, services may also internally
/// track additional state; this enum captures only the externally relevant
/// lifecycle as understood by the fabric.
///
/// # Lifecycle overview
///
/// - `Created`: service is registered but not initialized.
/// - `Initializing`: service is executing `init`.
/// - `Running`: service is operational.
/// - `Degraded`: service is operational but with reduced capability.
/// - `Stopping`: service is executing `stop`.
/// - `Stopped`: service has been gracefully stopped.
/// - `Failed`: service is considered failed; policies may decide whether and
///   how to recover.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServiceState {
    /// Service has been registered but not yet touched by the runtime.
    Created,
    /// Service is in its initialization phase.
    Initializing,
    /// Service is considered fully operational.
    Running,
    /// Service is running, but in a degraded mode.
    Degraded,
    /// Service is being stopped gracefully.
    Stopping,
    /// Service has been successfully stopped.
    Stopped,
    /// Service has irrecoverably failed.
    Failed,
}

impl ServiceState {
    /// Returns `true` if this state is considered terminal by default.
    ///
    /// The fabric runtime will typically avoid transitioning terminal services
    /// unless a recovery policy explicitly dictates otherwise.
    #[inline]
    pub const fn is_terminal(self) -> bool {
        matches!(self, ServiceState::Stopped | ServiceState::Failed)
    }
}

//! Recovery policies for services.
//!
//! Recovery is intentionally expressed in a small, allocation-free way so that
//! it can be used in early boot and `no_std` environments.
//!
//! The runtime applies policies in two main phases:
//!
//! - **Initialization failures**: errors returned from `Service::init` or `Service::start`
//! - **Runtime failures**: `Service::health` returns `HealthLevel::Failed`

/// Action to take when a service operation fails.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    /// Retry the operation a limited number of times.
    ///
    /// `max_retries = 0` means "do not retry".
    Retry { max_retries: u8 },
    /// Restart the service (best-effort stop, then init + start again).
    Restart,
    /// Mark the service as degraded but keep the system running.
    MarkDegraded,
    /// Fail fast: propagate the error and stop the system.
    Fail,
}

/// Per-service recovery policy.
///
/// Separate actions can be defined for initialization and runtime failures.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPolicy {
    /// What to do when initialization fails.
    pub action_on_init_failure: RecoveryAction,
    /// What to do when a runtime failure is detected.
    pub action_on_runtime_failure: RecoveryAction,
}

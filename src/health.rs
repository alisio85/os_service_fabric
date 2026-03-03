//! Health reporting primitives for services.
//!
//! `os_service_fabric` intentionally separates:
//!
//! - **Lifecycle state** (e.g. `Running`, `Stopped`) from
//! - **Health level** (e.g. `Ok`, `Degraded`, `Failed`)
//!
//! This separation is useful in OS kernels: a subsystem might still be
//! `Running` while reporting `Degraded` health, allowing the rest of the OS
//! to continue operating while acknowledging reduced functionality.

/// Coarse-grained health level for a service.
///
/// This intentionally mirrors, but is distinct from, the lifecycle state.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HealthLevel {
    /// Service is healthy.
    Ok,
    /// Service is operating with reduced capabilities.
    Degraded,
    /// Service is considered failed for health purposes.
    Failed,
}

/// An instantaneous health report produced by a service.
///
/// The report is kept intentionally small and allocation-free.
/// Optional diagnostic messages are represented as static strings.
///
/// # Examples
///
/// ```
/// use os_service_fabric::{HealthLevel, ServiceHealth};
///
/// let h = ServiceHealth::degraded(Some("low entropy; using fallback RNG"));
/// assert_eq!(h.level(), HealthLevel::Degraded);
/// assert_eq!(h.message(), Some("low entropy; using fallback RNG"));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ServiceHealth {
    level: HealthLevel,
    message: Option<&'static str>,
}

impl ServiceHealth {
    /// Returns a health report indicating full health.
    #[inline]
    pub const fn ok() -> Self {
        ServiceHealth {
            level: HealthLevel::Ok,
            message: None,
        }
    }

    /// Returns a health report indicating degraded health with an optional message.
    #[inline]
    pub const fn degraded(message: Option<&'static str>) -> Self {
        ServiceHealth {
            level: HealthLevel::Degraded,
            message,
        }
    }

    /// Returns a health report indicating failure with an optional message.
    #[inline]
    pub const fn failed(message: Option<&'static str>) -> Self {
        ServiceHealth {
            level: HealthLevel::Failed,
            message,
        }
    }

    /// Returns the current health level.
    #[inline]
    pub const fn level(&self) -> HealthLevel {
        self.level
    }

    /// Returns the associated static diagnostic message, if any.
    #[inline]
    pub const fn message(&self) -> Option<&'static str> {
        self.message
    }
}

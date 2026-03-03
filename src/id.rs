//! Service identifiers used by the fabric.

/// A stable, human-readable identifier for a service.
///
/// This is typically a `&'static str` constant, so it can be used freely
/// in `no_std` contexts without allocation.
///
/// # Examples
///
/// ```
/// use os_service_fabric::ServiceId;
///
/// const ID_SCHEDULER: ServiceId = ServiceId::new("scheduler");
///
/// assert_eq!(ID_SCHEDULER.as_str(), "scheduler");
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ServiceId {
    name: &'static str,
}

impl ServiceId {
    /// Creates a new service identifier from a static string.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        ServiceId { name }
    }

    /// Returns the underlying static string name of this service identifier.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        self.name
    }
}

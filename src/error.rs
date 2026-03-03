//! Error types used by the fabric and services.

/// Generic error type for service operations.
///
/// The fabric itself never allocates or performs I/O in order to create error
/// values. Services are free to define richer error enums and convert them
/// into this type if needed.
///
/// This type uses `&'static str` messages to remain allocation-free and
/// `no_std`-friendly.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ServiceError {
    /// The requested transition was not compatible with the current state.
    InvalidTransition(&'static str),
    /// A required dependency was not in a compatible state.
    Dependency(&'static str),
    /// An implementation-specific failure.
    Implementation(&'static str),
}

//! Common, internal use type definitions.

/// Generic, dyn boxed error type.
///
/// Alias for `Box<dyn Error>`.
pub(crate) type BoxErr = Box<dyn std::error::Error>;

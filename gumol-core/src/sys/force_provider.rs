// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! Trait for pluggable force computation (e.g. GPU acceleration).

use crate::{System, Vector3D};

/// Provider for force computation, allowing GPU or other accelerators to
/// override the default CPU path.
///
/// When set on a [`System`], [`System::forces`] will use this provider
/// instead of the default CPU implementation.
pub trait ForceProvider: Send + Sync {
    /// Compute forces for the system.
    ///
    /// The implementation may use GPU acceleration when the system is
    /// compatible, and fall back to CPU otherwise.
    fn forces(&self, system: &System) -> Vec<Vector3D>;
}

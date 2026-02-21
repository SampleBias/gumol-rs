use gumol_core::{System, Vector3D};
use std::sync::Arc;

pub mod dispatcher;
pub mod kernels;
pub mod memory;
pub mod profile;

pub use dispatcher::{check_gpu_compatibility, GpuCompatibility};
pub use memory::GpuSystemState;
pub use profile::{GpuProfile, GpuProfileBuilder};

use gumol_core::sys::compute::{Compute, Forces};

/// Error type for GPU operations
#[derive(Debug)]
pub enum GpuError {
    /// No CUDA device available
    NoDeviceAvailable,
    /// Device does not meet requirements
    DeviceNotSupported,
    /// GPU memory allocation failed
    MemoryAllocationFailed,
    /// Kernel launch failed
    KernelLaunchFailed,
    /// PTX compilation failed
    KernelCompilationFailed(String),
    /// CUDA driver error
    Driver(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::NoDeviceAvailable => write!(f, "No CUDA device available"),
            GpuError::DeviceNotSupported => write!(f, "Device not supported"),
            GpuError::MemoryAllocationFailed => write!(f, "GPU memory allocation failed"),
            GpuError::KernelLaunchFailed => write!(f, "Kernel launch failed"),
            GpuError::KernelCompilationFailed(s) => write!(f, "Kernel compilation failed: {s}"),
            GpuError::Driver(s) => write!(f, "CUDA driver error: {s}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Main GPU accelerator for Gumol
pub struct GpuAccelerator {
    device: Arc<cudarc::driver::CudaDevice>,
    profile: GpuProfile,
}

impl GpuAccelerator {
    /// Create a new GPU accelerator with default (GTX 1050) profile
    pub fn new() -> Result<Self, GpuError> {
        Self::with_profile(GpuProfile::default())
    }

    /// Create a new GPU accelerator with the specified profile
    pub fn with_profile(profile: GpuProfile) -> Result<Self, GpuError> {
        let device = cudarc::driver::CudaDevice::new(0)
            .map_err(|e| GpuError::Driver(e.to_string()))?;

        Ok(GpuAccelerator {
            device,
            profile,
        })
    }

    /// Check if GPU acceleration is available on this system
    pub fn is_available() -> bool {
        cudarc::driver::CudaDevice::new(0).is_ok()
    }

    /// Get device information
    pub fn device_info(&self) -> String {
        format!("CUDA device: ordinal {}", self.device.ordinal())
    }

    /// Get the GPU profile
    pub fn profile(&self) -> &GpuProfile {
        &self.profile
    }

    /// Set the GPU profile (e.g. when upgrading to a different GPU)
    pub fn set_profile(&mut self, profile: GpuProfile) {
        self.profile = profile;
    }

    /// Compute pair forces on GPU for a compatible system.
    ///
    /// Returns `Ok(Some(forces))` if GPU path was used, `Ok(None)` if system
    /// is not GPU-compatible (caller should use CPU), or `Err` on GPU failure.
    pub fn compute_pair_forces(&self, system: &System) -> Result<Option<Vec<Vector3D>>, GpuError> {
        let compat = check_gpu_compatibility(system, &self.profile);
        if !compat.compatible {
            return Ok(None);
        }

        let n_atoms = system.size();
        let positions: Vec<Vector3D> = system.particles().position.to_vec();
        let lengths = system.cell.lengths();
        let cell_lengths = [lengths[0], lengths[1], lengths[2]];

        let mut state = GpuSystemState::upload(&self.device, &positions, cell_lengths)
            .map_err(|e| GpuError::Driver(e.to_string()))?;

        kernels::launch_lj_forces(
            &self.device,
            &self.profile,
            &state.positions,
            &mut state.forces,
            compat.sigma,
            compat.epsilon,
            compat.cutoff,
            cell_lengths[0],
            cell_lengths[1],
            cell_lengths[2],
            n_atoms,
        )?;

        self.device
            .synchronize()
            .map_err(|e| GpuError::Driver(e.to_string()))?;

        let forces = state
            .download_forces(&self.device)
            .map_err(|e| GpuError::Driver(e.to_string()))?;

        Ok(Some(forces))
    }
}

impl Default for GpuAccelerator {
    fn default() -> Self {
        Self::new().expect("GPU not available")
    }
}

/// Force provider that uses GPU when possible, falls back to CPU otherwise.
///
/// Use with [`System::set_force_provider`] to enable GPU acceleration:
/// ```ignore
/// let accelerator = GpuAccelerator::with_profile(GpuProfile::gtx_1050())?;
/// system.set_force_provider(std::sync::Arc::new(GpuForceProvider::new(accelerator)));
/// ```
pub struct GpuForceProvider {
    accelerator: GpuAccelerator,
}

impl GpuForceProvider {
    /// Create a new GPU force provider
    pub fn new(accelerator: GpuAccelerator) -> Self {
        Self { accelerator }
    }

    /// Get the accelerator (e.g. to change profile for a different GPU)
    pub fn accelerator(&self) -> &GpuAccelerator {
        &self.accelerator
    }

    /// Get mutable accelerator
    pub fn accelerator_mut(&mut self) -> &mut GpuAccelerator {
        &mut self.accelerator
    }
}

impl gumol_core::ForceProvider for GpuForceProvider {
    fn forces(&self, system: &System) -> Vec<Vector3D> {
        match self.accelerator.compute_pair_forces(system) {
            Ok(Some(forces)) => forces,
            _ => Forces.compute(system),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_availability() {
        // This test will fail on systems without NVIDIA GPUs
        // It's okay to skip it in CI
        if GpuAccelerator::is_available() {
            let accel = GpuAccelerator::new().unwrap();
            let info = accel.device_info();
            assert!(!info.is_empty());
        }
    }
}

use gumol_core::*;

pub mod kernels;
pub mod memory;
pub mod dispatcher;

/// Error type for GPU operations
#[derive(Debug)]
pub enum GpuError {
    NoDeviceAvailable,
    DeviceNotSupported,
    MemoryAllocationFailed,
    KernelLaunchFailed,
}

/// Main GPU accelerator for Gumol
pub struct GpuAccelerator {
    device: cudarc::driver::CudaDevice,
}

impl GpuAccelerator {
    /// Create a new GPU accelerator
    ///
    /// Returns an error if no CUDA device is available
    pub fn new() -> Result<Self, GpuError> {
        let device = cudarc::driver::CudaDevice::new(0)
            .map_err(|_| GpuError::NoDeviceAvailable)?;

        Ok(GpuAccelerator { device })
    }

    /// Check if GPU acceleration is available on this system
    pub fn is_available() -> bool {
        cudarc::driver::CudaDevice::new(0).is_ok()
    }

    /// Get device information
    pub fn device_info(&self) -> String {
        format!(
            "CUDA device: {}",
            self.device.name()
        )
    }
}

impl Default for GpuAccelerator {
    fn default() -> Self {
        Self::new().expect("GPU not available")
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

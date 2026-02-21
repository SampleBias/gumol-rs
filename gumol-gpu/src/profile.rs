// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! GPU profile configuration for different NVIDIA architectures.
//!
//! Profiles optimize kernel launch parameters (block size, thresholds) for
//! specific GPU models. Use `GpuProfile::gtx_1050()` for GTX 1050 (Pascal),
//! `GpuProfile::rtx_3050_ti()` for RTX 3050 Ti (Turing), or create custom profiles.

/// GPU profile with architecture-specific optimization parameters.
#[derive(Clone, Debug)]
pub struct GpuProfile {
    /// Human-readable name (e.g. "GTX 1050", "RTX 3050 Ti")
    pub name: String,

    /// CUDA compute capability (e.g. 61 for Pascal, 75 for Turing)
    /// Used for PTX compilation: "compute_XX"
    pub compute_capability: u8,

    /// Thread block size for kernel launches (x dimension).
    /// Smaller values (128-256) for older GPUs with fewer cores.
    pub block_size: u32,

    /// Minimum number of atoms to use GPU path.
    /// Below this threshold, CPU overhead dominates.
    pub min_atoms_for_gpu: usize,

    /// Maximum atoms to allocate on GPU (memory limit).
    /// Approximate; actual limit depends on VRAM.
    pub max_atoms: usize,

    /// Use FP32 instead of FP64 for force computation (faster on consumer GPUs).
    /// Note: FP64 is slower on Pascal/consumer GPUs.
    pub prefer_fp32: bool,
}

impl GpuProfile {
    /// Profile optimized for GTX 1050 (Pascal, compute capability 6.1).
    /// 640 CUDA cores, 2-4 GB VRAM.
    pub fn gtx_1050() -> Self {
        Self {
            name: "GTX 1050".to_string(),
            compute_capability: 61,
            block_size: 128,
            min_atoms_for_gpu: 1000,
            max_atoms: 50_000,
            prefer_fp32: bool::default(),
        }
    }

    /// Profile optimized for GTX 1050 Ti (Pascal, compute capability 6.1).
    pub fn gtx_1050_ti() -> Self {
        Self {
            name: "GTX 1050 Ti".to_string(),
            compute_capability: 61,
            block_size: 128,
            min_atoms_for_gpu: 800,
            max_atoms: 50_000,
            prefer_fp32: bool::default(),
        }
    }

    /// Profile optimized for RTX 3050 Ti (Turing/Ampere, compute capability 7.5)
    pub fn rtx_3050_ti() -> Self {
        Self {
            name: "RTX 3050 Ti".to_string(),
            compute_capability: 75,
            block_size: 256,
            min_atoms_for_gpu: 500,
            max_atoms: 200_000,
            prefer_fp32: bool::default(),
        }
    }

    /// Profile for RTX 3060 and similar (Ampere, compute capability 8.6)
    pub fn rtx_3060() -> Self {
        Self {
            name: "RTX 3060".to_string(),
            compute_capability: 86,
            block_size: 256,
            min_atoms_for_gpu: 500,
            max_atoms: 500_000,
            prefer_fp32: bool::default(),
        }
    }

    /// Profile for RTX 4090 and high-end (Ada Lovelace, compute capability 8.9)
    pub fn rtx_4090() -> Self {
        Self {
            name: "RTX 4090".to_string(),
            compute_capability: 89,
            block_size: 256,
            min_atoms_for_gpu: 500,
            max_atoms: 1_000_000,
            prefer_fp32: bool::default(),
        }
    }

    /// Auto-detect profile from device compute capability.
    /// Falls back to GTX 1050 (Pascal) for unknown devices.
    pub fn from_compute_capability(major: i32, minor: i32) -> Self {
        let cc = (major * 10 + minor) as u8;
        match cc {
            61 => Self::gtx_1050(),
            75..=76 => Self::rtx_3050_ti(),
            86..=87 => Self::rtx_3060(),
            89.. => Self::rtx_4090(),
            _ => Self::gtx_1050(),
        }
    }

    /// Get the NVRTC architecture string for compilation (e.g. "compute_61")
    pub fn arch_string(&self) -> String {
        format!("compute_{}", self.compute_capability)
    }

    /// Create a custom profile with builder-style API.
    pub fn custom(name: impl Into<String>) -> GpuProfileBuilder {
        GpuProfileBuilder::new(name)
    }
}

/// Builder for custom GPU profiles.
#[derive(Clone, Debug)]
pub struct GpuProfileBuilder {
    name: String,
    compute_capability: u8,
    block_size: u32,
    min_atoms_for_gpu: usize,
    max_atoms: usize,
    prefer_fp32: bool,
}

impl GpuProfileBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compute_capability: 61,
            block_size: 128,
            min_atoms_for_gpu: 1000,
            max_atoms: 50_000,
            prefer_fp32: false,
        }
    }

    /// Set compute capability (e.g. 61 for Pascal)
    pub fn compute_capability(mut self, cc: u8) -> Self {
        self.compute_capability = cc;
        self
    }

    /// Set block size for kernel launches
    pub fn block_size(mut self, size: u32) -> Self {
        self.block_size = size;
        self
    }

    /// Set minimum atoms to use GPU
    pub fn min_atoms_for_gpu(mut self, n: usize) -> Self {
        self.min_atoms_for_gpu = n;
        self
    }

    /// Set maximum atoms (memory limit)
    pub fn max_atoms(mut self, n: usize) -> Self {
        self.max_atoms = n;
        self
    }

    /// Set FP32 preference
    pub fn prefer_fp32(mut self, f: bool) -> Self {
        self.prefer_fp32 = f;
        self
    }

    /// Build the profile
    pub fn build(self) -> GpuProfile {
        GpuProfile {
            name: self.name,
            compute_capability: self.compute_capability,
            block_size: self.block_size,
            min_atoms_for_gpu: self.min_atoms_for_gpu,
            max_atoms: self.max_atoms,
            prefer_fp32: self.prefer_fp32,
        }
    }
}

impl Default for GpuProfile {
    fn default() -> Self {
        Self::gtx_1050()
    }
}

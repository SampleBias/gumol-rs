// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! CUDA kernels for pair force computation.
//!
//! Lennard-Jones force kernel with periodic boundary conditions.

use cudarc::driver::{CudaDevice, CudaFunction, CudaSlice, DriverError, LaunchConfig};
use cudarc::nvrtc::{compile_ptx_with_opts, CompileOptions, Ptx};
use std::sync::Arc;

use crate::GpuError;
use super::profile::GpuProfile;

const LJ_KERNEL_SRC: &str = r#"
extern "C" __global__ void lj_forces(
    const double* __restrict__ positions,
    double* __restrict__ forces,
    const double sigma,
    const double epsilon,
    const double cutoff,
    const double cell_a,
    const double cell_b,
    const double cell_c,
    const int n_atoms
) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n_atoms) return;

    double xi = positions[3*i];
    double yi = positions[3*i+1];
    double zi = positions[3*i+2];

    double fxi = 0.0, fyi = 0.0, fzi = 0.0;

    for (int j = 0; j < n_atoms; j++) {
        if (i == j) continue;

        double dx = xi - positions[3*j];
        double dy = yi - positions[3*j+1];
        double dz = zi - positions[3*j+2];

        // Minimum image convention (orthorhombic)
        if (cell_a > 0.0) {
            dx -= rint(dx / cell_a) * cell_a;
        }
        if (cell_b > 0.0) {
            dy -= rint(dy / cell_b) * cell_b;
        }
        if (cell_c > 0.0) {
            dz -= rint(dz / cell_c) * cell_c;
        }

        double r2 = dx*dx + dy*dy + dz*dz;
        if (r2 >= cutoff*cutoff || r2 < 1e-20) continue;

        double r = sqrt(r2);
        double inv_r = 1.0 / r;
        double s_over_r = sigma * inv_r;
        double s6 = s_over_r * s_over_r * s_over_r * s_over_r * s_over_r * s_over_r;

        // LJ force: f = -24*eps*(s6 - 2*s12) / r, where s12 = s6*s6
        double s12 = s6 * s6;
        double force_mag = -24.0 * epsilon * (s6 - 2.0 * s12) * inv_r;

        fxi += force_mag * dx * inv_r;
        fyi += force_mag * dy * inv_r;
        fzi += force_mag * dz * inv_r;
    }

    forces[3*i]   = fxi;
    forces[3*i+1] = fyi;
    forces[3*i+2] = fzi;
}
"#;

/// Compile the LJ kernel PTX for the given profile.
fn compile_lj_kernel(profile: &GpuProfile) -> Result<Ptx, GpuError> {
    let arch = profile.arch_string();
    let arch_static: &'static str = Box::leak(arch.into_boxed_str());
    let opts = CompileOptions {
        arch: Some(arch_static),
        ..Default::default()
    };
    compile_ptx_with_opts(LJ_KERNEL_SRC, opts).map_err(|e| GpuError::KernelCompilationFailed(e.to_string()))
}

/// Load the LJ kernel into the device.
fn load_lj_kernel(device: &Arc<CudaDevice>, profile: &GpuProfile) -> Result<(), GpuError> {
    if device.has_func("gumol_lj", "lj_forces") {
        return Ok(());
    }
    let ptx = compile_lj_kernel(profile)?;
    device.load_ptx(ptx, "gumol_lj", &["lj_forces"])?;
    Ok(())
}

/// Launch the LJ forces kernel.
pub fn launch_lj_forces(
    device: &Arc<CudaDevice>,
    profile: &GpuProfile,
    positions: &CudaSlice<f64>,
    forces: &mut CudaSlice<f64>,
    sigma: f64,
    epsilon: f64,
    cutoff: f64,
    cell_a: f64,
    cell_b: f64,
    cell_c: f64,
    n_atoms: usize,
) -> Result<(), GpuError> {
    load_lj_kernel(device, profile)?;

    let func: CudaFunction = device
        .get_func("gumol_lj", "lj_forces")
        .ok_or(GpuError::KernelLaunchFailed)?;

    let block_size = profile.block_size;
    let grid_size = ((n_atoms as u32 + block_size - 1) / block_size).max(1);

    let cfg = LaunchConfig {
        grid_dim: (grid_size, 1, 1),
        block_dim: (block_size, 1, 1),
        shared_mem_bytes: 0,
    };

    unsafe {
        func.launch(
            cfg,
            (
                positions,
                forces,
                sigma,
                epsilon,
                cutoff,
                cell_a,
                cell_b,
                cell_c,
                n_atoms as i32,
            ),
        )
    }
    .map_err(|_| GpuError::KernelLaunchFailed)?;

    Ok(())
}

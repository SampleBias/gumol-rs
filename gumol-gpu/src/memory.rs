// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! GPU memory management for simulation data.
//!
//! Handles allocation and transfer of positions, forces, and potential
//! parameters between CPU and GPU.

use gumol_core::Vector3D;
use std::sync::Arc;

use cudarc::driver::{CudaDevice, CudaSlice, DriverError};

/// GPU-resident system state for LJ pair force computation.
///
/// Stores positions (3*N), forces (3*N), and homogeneous LJ parameters.
pub struct GpuSystemState {
    /// Number of atoms
    pub n_atoms: usize,

    /// Device positions: [x0,y0,z0, x1,y1,z1, ...] (3*N f64)
    pub positions: CudaSlice<f64>,

    /// Device forces: [fx0,fy0,fz0, fx1,fy1,fz1, ...] (3*N f64)
    pub forces: CudaSlice<f64>,

    /// Device cell lengths [a, b, c] for PBC
    pub cell_lengths: CudaSlice<f64>,
}

impl GpuSystemState {
    /// Allocate and upload system state to GPU.
    ///
    /// # Arguments
    /// * `device` - CUDA device
    /// * `positions` - Particle positions (N × 3)
    /// * `cell_lengths` - Unit cell lengths [a, b, c]
    pub fn upload(
        device: &Arc<CudaDevice>,
        positions: &[Vector3D],
        cell_lengths: [f64; 3],
    ) -> Result<Self, DriverError> {
        let n_atoms = positions.len();

        // Flatten positions to [x0,y0,z0, x1,y1,z1, ...]
        let pos_flat: Vec<f64> = positions
            .iter()
            .flat_map(|v| [v[0], v[1], v[2]])
            .collect();

        let positions_dev = device.htod_sync_copy(&pos_flat)?;
        let forces_dev = device.alloc_zeros::<f64>(3 * n_atoms)?;
        let cell_dev = device.htod_sync_copy(&cell_lengths)?;

        Ok(Self {
            n_atoms,
            positions: positions_dev,
            forces: forces_dev,
            cell_lengths: cell_dev,
        })
    }

    /// Download forces from GPU to host.
    pub fn download_forces(&self, device: &Arc<CudaDevice>) -> Result<Vec<Vector3D>, DriverError> {
        let flat = device.dtoh_sync_copy(&self.forces)?;
        let forces: Vec<Vector3D> = flat
            .chunks_exact(3)
            .map(|c| Vector3D::new(c[0], c[1], c[2]))
            .collect();
        Ok(forces)
    }

    /// Update positions on GPU (e.g. after integration step).
    pub fn update_positions(
        &mut self,
        device: &Arc<CudaDevice>,
        positions: &[Vector3D],
    ) -> Result<(), DriverError> {
        let pos_flat: Vec<f64> = positions
            .iter()
            .flat_map(|v| [v[0], v[1], v[2]])
            .collect();
        device.htod_sync_copy_into(&pos_flat, &mut self.positions)?;
        Ok(())
    }
}

// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! CPU/GPU dispatch logic for force computation.
//!
//! Determines when to use GPU vs CPU based on system characteristics
//! and GPU profile.

use gumol_core::System;

use crate::profile::GpuProfile;

/// Parameters extracted from a system for GPU compatibility check.
#[derive(Debug)]
pub struct GpuCompatibility {
    /// Whether the system can use the GPU path
    pub compatible: bool,
    /// Lennard-Jones sigma (if homogeneous LJ)
    pub sigma: f64,
    /// Lennard-Jones epsilon (if homogeneous LJ)
    pub epsilon: f64,
    /// Cutoff distance
    pub cutoff: f64,
    /// Reason for incompatibility (if not compatible)
    pub reason: Option<String>,
}

/// Check if a system is compatible with GPU acceleration.
///
/// Requirements:
/// - Homogeneous pair potential (single pair type)
/// - Lennard-Jones potential
/// - No bonded interactions (bonds, angles, dihedrals)
/// - No Coulomb/electrostatics
/// - No global potentials
/// - Periodic orthorhombic cell (or infinite)
/// - Particle count within profile limits
pub fn check_gpu_compatibility(system: &System, profile: &GpuProfile) -> GpuCompatibility {
    let n_atoms = system.size();

    if n_atoms < profile.min_atoms_for_gpu {
        return GpuCompatibility {
            compatible: false,
            sigma: 0.0,
            epsilon: 0.0,
            cutoff: 0.0,
            reason: Some(format!(
                "Too few atoms ({} < {})",
                n_atoms, profile.min_atoms_for_gpu
            )),
        };
    }

    if n_atoms > profile.max_atoms {
        return GpuCompatibility {
            compatible: false,
            sigma: 0.0,
            epsilon: 0.0,
            cutoff: 0.0,
            reason: Some(format!(
                "Too many atoms ({} > {})",
                n_atoms, profile.max_atoms
            )),
        };
    }

    // Check for bonded interactions
    for molecule in system.molecules() {
        if !molecule.bonds().is_empty()
            || !molecule.angles().is_empty()
            || !molecule.dihedrals().is_empty()
        {
            return GpuCompatibility {
                compatible: false,
                sigma: 0.0,
                epsilon: 0.0,
                cutoff: 0.0,
                reason: Some("System has bonded interactions (bonds/angles/dihedrals)".to_string()),
            };
        }
    }

    if system.coulomb_potential().is_some() {
        return GpuCompatibility {
            compatible: false,
            sigma: 0.0,
            epsilon: 0.0,
            cutoff: 0.0,
            reason: Some("System has Coulomb/electrostatics".to_string()),
        };
    }

    if !system.global_potentials().is_empty() {
        return GpuCompatibility {
            compatible: false,
            sigma: 0.0,
            epsilon: 0.0,
            cutoff: 0.0,
            reason: Some("System has global potentials".to_string()),
        };
    }

    // Check for homogeneous LJ - get pair potential from first pair
    let potential = system.pair_potential(0, 1);
    let potential = match potential {
        Some(p) => p,
        None => {
            return GpuCompatibility {
                compatible: false,
                sigma: 0.0,
                epsilon: 0.0,
                cutoff: 0.0,
                reason: Some("No pair potential defined".to_string()),
            };
        }
    };

    let (sigma, epsilon) = match potential.lj_params() {
        Some(params) => params,
        None => {
            return GpuCompatibility {
                compatible: false,
                sigma: 0.0,
                epsilon: 0.0,
                cutoff: 0.0,
                reason: Some("Pair potential is not Lennard-Jones".to_string()),
            };
        }
    };

    let cutoff = potential.cutoff();

    // Verify homogeneity: all pairs should use same potential
    let composition = system.composition();
    for (kind_i, _) in composition.all_particles() {
        for (kind_j, _) in composition.all_particles() {
            if let Some(p) = system.interactions().pair((kind_i, kind_j)) {
                if p.lj_params().is_none() {
                    return GpuCompatibility {
                        compatible: false,
                        sigma: 0.0,
                        epsilon: 0.0,
                        cutoff: 0.0,
                        reason: Some("Heterogeneous pair potentials".to_string()),
                    };
                }
            }
        }
    }

    GpuCompatibility {
        compatible: true,
        sigma,
        epsilon,
        cutoff,
        reason: None,
    }
}

// Gumol - GPU-Accelerated Radiation Simulation Engine
// Copyright (C) Gumol's contributors — BSD license

//! Molecular dynamics simulation of Argon with optional GPU acceleration.
//!
//! Run with: `cargo run --example argon_gpu --features gpu`
//! Requires CUDA toolkit and NVIDIA GPU.

#![allow(clippy::cast_lossless)]

use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

use gumol_sim::output::{EnergyOutput, TrajectoryOutput};
use gumol_sim::{MolecularDynamics, Simulation};
use gumol_sim::{BoltzmannVelocities, InitVelocities};

#[cfg(feature = "gpu")]
use gumol_gpu::{GpuAccelerator, GpuForceProvider, GpuProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    // Create a cubic crystal of Argon (10^3 = 1000 atoms for GPU threshold)
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                let position = Vector3D::new(i as f64 * 3.4, j as f64 * 3.4, k as f64 * 3.4);
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
            }
        }
    }

    let lj = Box::new(LennardJones {
        sigma: units::from(3.4, "A")?,
        epsilon: units::from(1.0, "kJ/mol")?,
    });
    system.set_pair_potential(
        ("Ar", "Ar"),
        PairInteraction::new(lj, units::from(8.5, "A")?),
    );

    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);

    // Enable GPU acceleration when available
    #[cfg(feature = "gpu")]
    {
        if GpuAccelerator::is_available() {
            // Use GTX 1050 profile - change to GpuProfile::rtx_3050_ti() for newer GPUs
            let profile = GpuProfile::gtx_1050();
            let accelerator = GpuAccelerator::with_profile(profile)?;
            println!("GPU: {} ({})", accelerator.device_info(), accelerator.profile().name);
            system.set_force_provider(std::sync::Arc::new(GpuForceProvider::new(accelerator)));
        } else {
            println!("GPU not available, using CPU");
        }
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("Build with --features gpu for GPU acceleration");
    }

    let md = MolecularDynamics::new(units::from(1.0, "fs")?);
    let mut simulation = Simulation::new(Box::new(md));

    let trajectory_out = Box::new(TrajectoryOutput::new("trajectory.xyz")?);
    simulation.add_output_with_frequency(trajectory_out, 10);

    let energy_out = Box::new(EnergyOutput::new("energy.dat")?);
    simulation.add_output(energy_out);

    simulation.run(&mut system, 5000);

    println!("All done!");

    Ok(())
}

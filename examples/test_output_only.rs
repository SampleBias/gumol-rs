// Test simulation step-by-step to find SIGFPE
#![allow(clippy::cast_lossless)]

use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

use gumol_sim::{BoltzmannVelocities, InitVelocities};
use gumol_sim::output::{EnergyOutput, TrajectoryOutput, Output};

#[cfg(feature = "gpu")]
use gumol_gpu::{GpuAccelerator, GpuForceProvider, GpuProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    // Create a cubic crystal of Argon (27 atoms)
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                let position = Vector3D::new(i as f64 * 3.4, j as f64 * 3.4, k as f64 * 3.4);
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
            }
        }
    }

    println!("Created system with {} atoms", system.size());

    let lj = Box::new(LennardJones {
        sigma: units::from(3.4, "A")?,
        epsilon: units::from(1.0, "kJ/mol")?,
    });
    system.set_pair_potential(
        ("Ar", "Ar"),
        PairInteraction::new(lj, units::from(8.5, "A")?),
    );

    println!("Setting velocities...");
    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);
    println!("Velocities initialized successfully");
    println!("Temperature after init: {}", system.temperature());

    // Enable GPU acceleration when available
    #[cfg(feature = "gpu")]
    {
        if GpuAccelerator::is_available() {
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

    // Test writing trajectory first, before any simulation
    println!("\nTesting trajectory write before simulation...");
    let mut trajectory_out = TrajectoryOutput::new("trajectory_test.xyz")?;
    trajectory_out.setup(&system);
    trajectory_out.write(&system);
    trajectory_out.finish(&system);
    println!("Trajectory write succeeded");

    // Test energy output
    println!("\nTesting energy output before simulation...");
    let mut energy_out = EnergyOutput::new("energy_test.dat")?;
    energy_out.setup(&system);
    energy_out.write(&system);
    energy_out.finish(&system);
    println!("Energy output succeeded");

    println!("\nAll tests passed!");

    Ok(())
}

// Test GPU force computation in isolation
#![allow(clippy::cast_lossless)]

use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

#[cfg(feature = "gpu")]
use gumol_gpu::{GpuAccelerator, GpuForceProvider, GpuProfile};
use gumol_sim::{BoltzmannVelocities, InitVelocities};

#[cfg(feature = "gpu")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    // Create a cubic crystal of Argon (1000 atoms like original argon_gpu)
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
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

    // Initialize velocities
    use gumol_sim::{BoltzmannVelocities, InitVelocities};
    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);
    println!("Velocities initialized");

    // Set up GPU
    if GpuAccelerator::is_available() {
        let profile = GpuProfile::gtx_1050();
        let accelerator = GpuAccelerator::with_profile(profile)?;
        println!("GPU: {} ({})", accelerator.device_info(), accelerator.profile().name);
        system.set_force_provider(std::sync::Arc::new(GpuForceProvider::new(accelerator)));

        println!("Computing forces with GPU...");
        let forces = system.forces();
        println!("GPU force computation complete");
        println!("Force on first particle: {:?}", forces[0]);
        println!("Force magnitude on first particle: {}", forces[0].norm());
    } else {
        println!("GPU not available");
    }

    Ok(())
}

#[cfg(not(feature = "gpu"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please run with --features gpu");
    Ok(())
}

// Test trajectory without velocities
#![allow(clippy::cast_lossless)]

use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

use gumol_sim::output::{EnergyOutput, Output};

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

    // Test with energy output
    println!("\nTesting energy output...");
    let mut energy_out = EnergyOutput::new("energy_no_sim.dat")?;
    energy_out.setup(&system);
    energy_out.write(&system);
    energy_out.finish(&system);
    println!("Energy output succeeded");

    println!("\nAll tests passed!");

    Ok(())
}

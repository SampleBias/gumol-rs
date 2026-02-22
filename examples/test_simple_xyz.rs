// Test simple XYZ output (bypasses chemfiles)
#![allow(clippy::cast_lossless)]

use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

use gumol_sim::{BoltzmannVelocities, InitVelocities};
use gumol_sim::{MolecularDynamics, Simulation};
use gumol_sim::output::{EnergyOutput, Output, SimpleXYZOutput};

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

    let md = MolecularDynamics::new(units::from(1.0, "fs")?);
    let mut simulation = Simulation::new(Box::new(md));

    // Use SimpleXYZOutput instead of TrajectoryOutput to avoid chemfiles SIGFPE
    let trajectory_out = Box::new(SimpleXYZOutput::new("trajectory_simple.xyz")?);
    simulation.add_output_with_frequency(trajectory_out, 10);

    let energy_out = Box::new(EnergyOutput::new("energy_simple.dat")?);
    simulation.add_output(energy_out);

    println!("Starting simulation...");
    simulation.run(&mut system, 100);
    println!("Simulation complete!");

    println!("Final temperature: {}", system.temperature());
    println!("Final energy: E_pot={}, E_kin={}", system.potential_energy(), system.kinetic_energy());

    Ok(())
}

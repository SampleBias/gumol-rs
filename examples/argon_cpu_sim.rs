// CPU-only simulation test
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::output::{EnergyOutput, TrajectoryOutput};
use gumol_sim::{MolecularDynamics, Simulation};
use gumol_sim::{BoltzmannVelocities, InitVelocities};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

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

    println!("Temperature after init: {}", system.temperature());

    let md = MolecularDynamics::new(units::from(1.0, "fs")?);
    let mut simulation = Simulation::new(Box::new(md));

    let trajectory_out = Box::new(TrajectoryOutput::new("trajectory_cpu.xyz")?);
    simulation.add_output_with_frequency(trajectory_out, 100);

    let energy_out = Box::new(EnergyOutput::new("energy_cpu.dat")?);
    simulation.add_output(energy_out);

    println!("Running 100 steps...");
    simulation.run(&mut system, 100);

    println!("Done!");

    Ok(())
}

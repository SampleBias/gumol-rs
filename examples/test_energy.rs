// Minimal simulation test with output
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::output::EnergyOutput;
use gumol_sim::{MolecularDynamics, Simulation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
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

    println!("Step 0: Checking system...");
    system.check();

    let energy_out = Box::new(EnergyOutput::new("energy_minimal.dat")?);

    println!("Step 1: Writing energy...");
    energy_out.setup(&system);
    energy_out.write(&system);

    println!("Step 2: Computing potential energy...");
    let pe = system.potential_energy();
    println!("Potential energy: {}", pe);

    println!("All done!");

    Ok(())
}

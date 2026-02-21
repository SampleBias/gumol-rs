// Minimal test
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::{BoltzmannVelocities, InitVelocities};

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

    println!("Size: {}", system.size());
    println!("Cell: {:?}", system.cell);

    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    println!("Initializing velocities...");
    velocities.init(&mut system);

    println!("Temperature: {}", system.temperature());
    println!("Kinetic energy: {}", system.kinetic_energy());

    Ok(())
}

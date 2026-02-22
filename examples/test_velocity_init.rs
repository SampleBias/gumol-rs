// Test velocity initialization in isolation
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::{BoltzmannVelocities, InitVelocities};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 1: 1000 atoms (original argon_gpu config)");
    test_system(1000, 10)?;

    println!("\nTest 2: 27 atoms");
    test_system(27, 3)?;

    println!("\nTest 3: 125 atoms");
    test_system(125, 5)?;

    println!("\nTest 4: 216 atoms");
    test_system(216, 6)?;

    println!("\nAll tests passed!");

    Ok(())
}

fn test_system(n_atoms_per_side: usize, grid_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    // Create a cubic crystal of Argon
    for i in 0..grid_size {
        for j in 0..grid_size {
            for k in 0..grid_size {
                let position = Vector3D::new(i as f64 * 3.4, j as f64 * 3.4, k as f64 * 3.4);
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
            }
        }
    }

    let actual_count = system.size();
    println!("  Created system with {} atoms", actual_count);

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

    let temp = system.temperature();
    println!("  Temperature after init: {}", temp);

    // Try to compute some energies
    let potential = system.potential_energy();
    let kinetic = system.kinetic_energy();
    println!("  Potential energy: {}", potential);
    println!("  Kinetic energy: {}", kinetic);

    Ok(())
}

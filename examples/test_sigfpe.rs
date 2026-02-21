// Simple test program to debug SIGFPE
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating system...");
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    println!("Adding particles...");
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                let position = Vector3D::new(i as f64 * 3.4, j as f64 * 3.4, k as f64 * 3.4);
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
            }
        }
    }

    println!("Setting potential...");
    let lj = Box::new(LennardJones {
        sigma: units::from(3.4, "A")?,
        epsilon: units::from(1.0, "kJ/mol")?,
    });
    system.set_pair_potential(
        ("Ar", "Ar"),
        PairInteraction::new(lj, units::from(8.5, "A")?),
    );

    println!("System size: {}", system.size());
    println!("Cell volume: {}", system.cell.volume());
    println!("Degrees of freedom: {}", system.degrees_of_freedom());

    println!("Computing forces...");
    let forces = system.forces();
    println!("Computed {} forces", forces.len());

    println!("Computing temperature...");
    let temperature = system.temperature();
    println!("Temperature: {}", temperature);

    println!("Computing potential energy...");
    let pe = system.potential_energy();
    println!("Potential energy: {}", pe);

    println!("All done!");

    Ok(())
}

// Direct chemfiles test
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::{BoltzmannVelocities, InitVelocities};
use chemfiles;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    // Create 27 atoms
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

    // Initialize velocities
    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);
    println!("Velocities initialized");

    // Test writing trajectory using chemfiles directly, via System::into conversion
    println!("\nTesting chemfiles frame creation with velocities...");
    let mut trajectory = chemfiles::Trajectory::open("test_chemfiles.xyz", 'w')?;

    // Try to convert system to chemfiles::Frame
    println!("Converting system to frame...");
    let frame: chemfiles::Frame = (&system).into();
    println!("Frame created successfully");

    // Try to write the frame
    println!("Writing frame to trajectory...");
    trajectory.write(&mut frame)?;
    println!("Frame written successfully");

    println!("\nAll tests passed!");

    Ok(())
}

// Test with velocity init and controls
use gumol_core::{Particle, Molecule, System, UnitCell, Vector3D};
use gumol_core::energy::{LennardJones, PairInteraction};
use gumol_core::units;
use gumol_sim::md::VelocityVerlet;
use gumol_sim::md::Integrator;
use gumol_sim::md::Control;
use gumol_sim::{BoltzmannVelocities, InitVelocities};
use gumol_sim::md::RemoveTranslation;
use gumol_sim::md::RemoveRotation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = System::with_cell(UnitCell::cubic(35.0));

    let mut count = 0;
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                if count >= 1000 {
                    break;
                }
                let position = Vector3D::new(i as f64 * 3.4, j as f64 * 3.4, k as f64 * 3.4);
                let particle = Particle::with_position("Ar", position);
                system.add_molecule(Molecule::new(particle));
                count += 1;
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

    println!("Step 1: Initializing velocities...");
    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K")?);
    velocities.seed(129);
    velocities.init(&mut system);
    println!("  Temp after init: {}", system.temperature());

    println!("Step 2: Running controls...");
    let mut rm_trans = RemoveTranslation;
    let mut rm_rot = RemoveRotation;
    println!("  Removing translation...");
    rm_trans.control(&mut system);
    println!("  Temp after trans: {}", system.temperature());
    println!("  Removing rotation...");
    rm_rot.control(&mut system);
    println!("  Temp after rot: {}", system.temperature());

    println!("Step 3: Running 20 MD steps...");
    let mut integrator = VelocityVerlet::new(units::from(1.0, "fs")?);
    integrator.setup(&system);

    for step in 1..=20 {
        integrator.integrate(&mut system);
        if step % 5 == 0 {
            println!("Step {}: Temp: {}, PE: {}", step, system.temperature(), system.potential_energy());
        }
    }

    println!("All done!");

    Ok(())
}

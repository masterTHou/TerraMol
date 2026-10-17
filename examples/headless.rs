use mol_engine_core::{chemistry::template, Simulation};
use std::sync::Arc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = Simulation::native(100., 1.)?;
    sim.world
        .batch_spawn_molecules(Arc::new(template(0)?), 50, 42, 1000)?;
    sim.world.apply_thermal_velocities(300., 42)?;
    sim.step(1000)?;
    println!(
        "{} molecules, {} atoms, state {}",
        sim.world.molecules().len(),
        sim.world.atoms().len(),
        sim.world.state_version
    );
    Ok(())
}

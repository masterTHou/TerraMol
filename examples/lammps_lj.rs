#[cfg(feature = "lammps")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use mol_engine_core::{
        chemistry::parse_xyz,
        physics::{
            lammps_backend::{LammpsBackend, LennardJones},
            PhysicsBackend,
        },
        world::World,
    };
    use std::sync::Arc;
    let path = std::env::var("LAMMPS_LIBRARY")?;
    let def = Arc::new(parse_xyz(
        "1\nGeneric LJ particle (C mass; not a carbon force field)\nC 0 0 0",
    )?);
    let mut w = World::new(40.)?;
    w.spawn_molecule(def.clone(), [17., 20., 20.], [0., 0., 0., 1.])?;
    w.spawn_molecule(def, [23., 20., 20.], [0., 0., 0., 1.])?;
    let mut backend = LammpsBackend::new(
        path,
        LennardJones {
            epsilon_kcal_mol: 0.1,
            sigma_angstrom: 3.4,
            cutoff_angstrom: 8.5,
        },
    )?;
    backend.initialize(&w, 0.1)?;
    let e0 = backend.total_energy()?;
    backend.step(&mut w, 0.1, 1000)?;
    println!(
        "E0={e0:.12}, E1={:.12} kcal/mol; x={:?}",
        backend.total_energy()?,
        w.atoms()[0].position
    );
    Ok(())
}
#[cfg(not(feature = "lammps"))]
fn main() {
    eprintln!("Use cargo run --features lammps --example lammps_lj");
}

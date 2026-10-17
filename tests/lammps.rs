#![cfg(feature = "lammps")]
use mol_engine_core::{
    chemistry::{parse_xyz, template},
    physics::{
        lammps_backend::{LammpsBackend, LennardJones},
        PhysicsBackend,
    },
    world::World,
    Simulation,
};
use std::sync::Arc;
fn backend() -> LammpsBackend {
    LammpsBackend::new(
        std::env::var("LAMMPS_LIBRARY").expect("set absolute LAMMPS_LIBRARY"),
        LennardJones {
            epsilon_kcal_mol: 0.1,
            sigma_angstrom: 3.4,
            cutoff_angstrom: 8.5,
        },
    )
    .unwrap()
}
#[test]
#[ignore = "requires serial LAMMPS_LIBRARY"]
fn lj_conservation_mapping_reproducibility() {
    fn run() -> [f64; 3] {
        let def = Arc::new(parse_xyz("1\ntoy LJ C\nC 0 0 0").unwrap());
        let mut w = World::new(40.).unwrap();
        let a = w
            .spawn_molecule(def.clone(), [17., 20., 20.], [0., 0., 0., 1.])
            .unwrap();
        w.spawn_molecule(def, [23., 20., 20.], [0., 0., 0., 1.])
            .unwrap();
        let mut l = backend();
        l.initialize(&w, 0.1).unwrap();
        let e = l.total_energy().unwrap();
        l.step(&mut w, 0.1, 1000).unwrap();
        assert!(w.atoms()[0].position[0] > 17.);
        assert!((l.total_energy().unwrap() - e).abs() < 1e-7);
        let position = w.atoms()[0].position;
        w.remove_molecule(a).unwrap();
        l.sync_topology(&w).unwrap();
        w.set_molecule_velocity(w.molecules()[0].id, [0.001, 0., 0.])
            .unwrap();
        l.sync_state(&w).unwrap();
        let x = w.atoms()[0].position[0];
        l.step(&mut w, 0.1, 10).unwrap();
        assert!((w.atoms()[0].position[0] - x - 0.001).abs() < 1e-9);
        w.clear_all();
        l.sync_topology(&w).unwrap();
        l.step(&mut w, 0.1, 100).unwrap();
        position
    }
    assert_eq!(run(), run());
}
#[test]
fn rejects_bonded_templates_without_loading() {
    let mut w = World::new(40.).unwrap();
    w.spawn_molecule(Arc::new(template(0).unwrap()), [20.; 3], [0., 0., 0., 1.])
        .unwrap();
    let l = LammpsBackend::new(
        "missing",
        LennardJones {
            epsilon_kcal_mol: 0.1,
            sigma_angstrom: 3.4,
            cutoff_angstrom: 8.5,
        },
    )
    .unwrap();
    assert!(Simulation::new(w, 1., Box::new(l)).is_err());
}
#[test]
#[ignore = "requires serial LAMMPS_LIBRARY"]
fn ffi_lj_lifecycle() {
    unsafe {
        use mol_engine_core::ffi::*;
        let path = std::env::var("LAMMPS_LIBRARY").unwrap();
        let mut h = 0;
        assert_eq!(
            me_create_lj(
                path.as_ptr(),
                path.len() as u32,
                40.,
                0.1,
                0.1,
                3.4,
                8.5,
                &mut h
            ),
            0
        );
        let mut a = 0;
        let mut b = 0;
        assert_eq!(me_spawn_particle(h, 6, 17., 20., 20., &mut a), 0);
        assert_eq!(me_spawn_particle(h, 6, 23., 20., 20., &mut b), 0);
        assert_eq!(me_step(h, 1000), 0);
        let mut n = 0;
        assert_eq!(me_snapshot_size(h, &mut n), 0);
        assert_eq!(n, 120);
        assert_eq!(me_remove(h, a), 0);
        assert_eq!(me_set_velocity(h, b, 0.001, 0., 0.), 0);
        assert_eq!(me_step(h, 10), 0);
        assert_eq!(me_clear(h), 0);
        assert_eq!(me_step(h, 1), 0);
        assert_eq!(me_destroy(h), 0);
    }
}

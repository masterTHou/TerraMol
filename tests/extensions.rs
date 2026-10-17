use mol_engine_core::{
    actions::Action,
    chemistry::{parse_xyz, template},
    conformer::{dihedral, Conformer, EnergyEstimate},
    environment::{ResetSpec, SpawnBatch},
    geometry::*,
    Simulation,
};
use std::sync::Arc;
#[test]
fn deterministic_reset_and_actions() {
    let spec = ResetSpec {
        batches: vec![SpawnBatch {
            definition: Arc::new(template(0).unwrap()),
            count: 50,
            seed: 42,
            max_retries: 1000,
        }],
        thermal_kelvin: 300.,
        thermal_seed: 9,
    };
    let mut sim = Simulation::native(100., 1.).unwrap();
    sim.reset(spec.clone()).unwrap();
    let first = sim.observe().unwrap();
    let old = first.molecules[0].id;
    sim.act(Action::Step(100)).unwrap();
    sim.reset(spec.clone()).unwrap();
    let second = sim.observe().unwrap();
    assert_eq!(second.step_count, 0);
    assert!(second.snapshot.header.topology_version > first.snapshot.header.topology_version);
    for (a, b) in first.snapshot.atoms.iter().zip(&second.snapshot.atoms) {
        assert_eq!(a.position, b.position);
        assert_eq!(a.velocity, b.velocity);
        assert_ne!(a.molecule_id, b.molecule_id);
    }
    assert!(sim.act(Action::Remove(old)).is_err());
    let mut invalid = spec;
    invalid.thermal_kelvin = -1.;
    let version = sim.world.topology_version();
    assert!(sim.reset(invalid).is_err());
    assert_eq!(sim.world.topology_version(), version);
    let o = sim.act(Action::Step(5)).unwrap();
    assert_eq!(o.time_fs, 5.);
}
#[test]
fn dihedral_cut_and_geometry() {
    let geometry = parse_xyz("4\nchain\nC 1 0 0\nC 0 0 0\nC 0 0 1\nC 1 0 1").unwrap();
    // Explicit test graph: XYZ candidates are not an authoritative topology.
    let bonds = vec![
        mol_engine_core::molecule::Bond {
            a: 0,
            b: 1,
            order: Some(1),
        },
        mol_engine_core::molecule::Bond {
            a: 1,
            b: 2,
            order: Some(1),
        },
        mol_engine_core::molecule::Bond {
            a: 2,
            b: 3,
            order: Some(1),
        },
    ];
    let d = mol_engine_core::molecule::MoleculeDefinition::new(
        "chain",
        geometry.atoms().to_vec(),
        bonds.clone(),
    )
    .unwrap();
    let p: Vec<_> = d.atoms().iter().map(|a| a.local).collect();
    assert!(dihedral::measure(&p, [0, 1, 2, 3]).unwrap().abs() < 1e-12);
    let q = dihedral::set(&d, &p, [0, 1, 2, 3], 1.2).unwrap();
    assert!((dihedral::measure(&q, [0, 1, 2, 3]).unwrap() - 1.2).abs() < 1e-12);
    for b in d.bonds() {
        assert!((norm(sub(p[b.a], p[b.b])) - norm(sub(q[b.a], q[b.b]))).abs() < 1e-12);
    }
    assert_eq!(p[0], q[0]);
    assert!(dihedral::measure(&p, [0, 0, 2, 3]).is_err());
    let mut ring_bonds = bonds;
    ring_bonds.push(mol_engine_core::molecule::Bond {
        a: 3,
        b: 0,
        order: Some(1),
    });
    let ring =
        mol_engine_core::molecule::MoleculeDefinition::new("ring", d.atoms().to_vec(), ring_bonds)
            .unwrap();
    assert!(dihedral::set(&ring, &p, [0, 1, 2, 3], 1.).is_err());
}
#[test]
fn energy_requires_provenance() {
    let d = template(0).unwrap();
    let p = d.atoms().iter().map(|a| a.local).collect::<Vec<_>>();
    assert!(Conformer::new(&d, p.clone(), None, Default::default()).is_ok());
    assert!(Conformer::new(
        &d,
        p,
        Some(EnergyEstimate {
            kcal_per_mol: 0.,
            method: "".into(),
            parameterization: None
        }),
        Default::default()
    )
    .is_err());
}

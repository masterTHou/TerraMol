use mol_engine_core::{actions::KB_VELOCITY, chemistry::*, geometry::*, world::World, Simulation};
use std::sync::Arc;
#[test]
fn selective_thermal_preserves_existing_motion() {
    let mut w = World::new(40.).unwrap();
    let def = Arc::new(template(0).unwrap());
    let ids = w.batch_spawn_molecules(def, 2, 42, 100).unwrap();
    w.set_molecule_velocity(ids[0], [0.01, 0., 0.]).unwrap();
    w.apply_thermal_velocities_for(&[ids[1]], 300., 42).unwrap();
    assert_eq!(w.molecules()[0].velocity, [0.01, 0., 0.]);
    assert!(norm(w.molecules()[1].velocity) > 0.);
    let before = w.molecules()[0].velocity;
    assert!(w
        .apply_thermal_velocities_for(&[ids[0], u64::MAX], 500., 42)
        .is_err());
    assert_eq!(w.molecules()[0].velocity, before);
}
#[test]
fn box_and_step_limits() {
    let d = Arc::new(parse_xyz("1\nH\nH 0 0 0").unwrap());
    let mut w = World::new(2.4).unwrap();
    assert!(w.spawn_molecule(d, [1.2; 3], [0., 0., 0., 1.]).is_err());
    let mut s = Simulation::native(100., 1.).unwrap();
    let id = s
        .world
        .spawn_molecule(Arc::new(template(0).unwrap()), [50.; 3], [0., 0., 0., 1.])
        .unwrap();
    s.world.set_molecule_velocity(id, [1e12, 0., 0.]).unwrap();
    assert!(s.step(1).is_err());
    assert!(s.world.atoms().iter().all(|a| finite(a.position)));
}
#[test]
fn templates() {
    for (k, n, b) in [(0, 3, 2), (1, 5, 4), (2, 12, 12)] {
        let m = template(k).unwrap();
        assert_eq!(m.atoms().len(), n);
        assert_eq!(m.bonds().len(), b);
        for bond in m.bonds() {
            let length = norm(sub(m.atoms()[bond.a].local, m.atoms()[bond.b].local));
            let target = match k {
                0 => 0.9572,
                1 => 1.09,
                _ => {
                    if bond.b < 6 {
                        1.397
                    } else {
                        1.09
                    }
                }
            };
            assert!((length - target).abs() < 1e-5);
        }
        assert!(
            norm(
                m.atoms()
                    .iter()
                    .fold([0.; 3], |v, a| add(v, scale(a.local, a.element.mass)))
            ) < 1e-12
        );
    }
}
#[test]
fn xyz() {
    let m = parse_xyz("3\nwater\nO 0 0 0\nH .9572 0 0\nH -.24 .927 0\n").unwrap();
    assert_eq!(m.bonds().len(), 2);
    assert!(m.bonds().iter().all(|b| b.order.is_none()));
    for s in [
        "",
        "0\nx",
        "2\nx\nH 0 0 0",
        "1\nx\nX 0 0 0",
        "1\nx\nH NaN 0 0",
        "1\nx\nH 0 0 0\nH 1 1 1",
    ] {
        assert!(parse_xyz(s).is_err());
    }
}
#[test]
fn batch_delete() {
    let mut w = World::new(100.).unwrap();
    let def = Arc::new(template(0).unwrap());
    let ids = w.batch_spawn_molecules(def.clone(), 50, 42, 100).unwrap();
    for a in w.molecules() {
        assert!(Arc::ptr_eq(&a.definition, &def));
        for b in w.molecules() {
            if a.id != b.id {
                assert!(
                    norm(sub(a.center, b.center)) >= a.definition.radius() + b.definition.radius()
                );
            }
        }
    }
    w.remove_molecule(ids[15]).unwrap();
    assert_eq!(w.atoms().len(), 147);
    assert!(w.bonds().iter().all(|b| b.a < 147 && b.b < 147));
    assert!(w
        .bonds()
        .iter()
        .all(|b| w.atoms()[b.a].molecule_id == w.atoms()[b.b].molecule_id));
    assert!(!w.atoms().iter().any(|a| a.molecule_id == ids[15]));
    let before = w.molecules().len();
    assert!(w.batch_spawn_molecules(def, 1000, 4, 1).is_err());
    assert_eq!(w.molecules().len(), before);
}
#[test]
fn rigid_reflection_reproducible() {
    fn run() -> Simulation {
        let mut s = Simulation::native(30., 1.).unwrap();
        let id = s
            .world
            .spawn_molecule(
                Arc::new(template(0).unwrap()),
                [25., 15., 15.],
                [0., 0., 0., 1.],
            )
            .unwrap();
        s.world.set_molecule_velocity(id, [0.2, 0., 0.]).unwrap();
        let d = norm(sub(
            s.world.atoms()[0].position,
            s.world.atoms()[1].position,
        ));
        s.step(1000).unwrap();
        assert!(
            (norm(sub(
                s.world.atoms()[0].position,
                s.world.atoms()[1].position
            )) - d)
                .abs()
                < 1e-12
        );
        assert!(s.world.molecules()[0].center[0] >= s.world.molecules()[0].definition.radius());
        s
    }
    assert_eq!(
        run().world.atoms()[0].position,
        run().world.atoms()[0].position
    );
}
#[test]
fn seed_and_temperature() {
    let mut w = World::new(1000.).unwrap();
    for k in 0..2 {
        w.batch_spawn_molecules(Arc::new(template(k).unwrap()), 4000, 123 + k as u64, 100)
            .unwrap();
    }
    w.apply_thermal_velocities(300., 9).unwrap();
    let expected: Vec<_> = w.molecules().iter().map(|m| m.velocity).collect();
    w.apply_thermal_velocities(300., 9).unwrap();
    assert_eq!(
        expected,
        w.molecules().iter().map(|m| m.velocity).collect::<Vec<_>>()
    );
    for range in [0..4000, 4000..8000] {
        let t = w.molecules()[range]
            .iter()
            .map(|m| m.definition.mass() * dot(m.velocity, m.velocity) / (3. * KB_VELOCITY))
            .sum::<f64>()
            / 4000.;
        assert!((t / 300. - 1.).abs() < 0.04, "{t}");
    }
}
#[test]
fn elastic_pair() {
    let mut s = Simulation::native(100., 1.).unwrap();
    let d = Arc::new(template(1).unwrap());
    let a = s
        .world
        .spawn_molecule(d.clone(), [40., 50., 50.], [0., 0., 0., 1.])
        .unwrap();
    let b = s
        .world
        .spawn_molecule(d, [60., 50., 50.], [0., 0., 0., 1.])
        .unwrap();
    s.world.set_molecule_velocity(a, [0.1, 0., 0.]).unwrap();
    s.world.set_molecule_velocity(b, [-0.1, 0., 0.]).unwrap();
    s.step(100).unwrap();
    assert!(s.world.molecules()[0].velocity[0] < 0.);
    assert!((s.world.molecules()[0].velocity[0] + 0.1).abs() < 1e-12);
}

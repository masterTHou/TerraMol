use mol_engine_core::{
    chemistry::{parse_xyz, template},
    geometry::*,
    physics::observables::Observables,
    Simulation,
};
use std::sync::Arc;
#[test]
fn inertia_matches_direct_tensor_and_singular_axes() {
    for kind in 0..3 {
        let d = template(kind).unwrap();
        let x = [0.3, -0.7, 1.2];
        let l = d.atoms().iter().fold([0.; 3], |s, a| {
            add(s, scale(cross(a.local, cross(x, a.local)), a.element.mass))
        });
        assert!(norm(sub(d.inertia.inverse(l), x)) < 1e-12);
        assert_eq!(d.inertia.dof(), 3);
    }
    let point = parse_xyz("1\npoint\nC 0 0 0").unwrap();
    assert_eq!(point.inertia.dof(), 0);
    let line = parse_xyz("2\nlinear\nH -1 0 0\nH 1 0 0").unwrap();
    assert_eq!(line.inertia.dof(), 2);
    assert_eq!(line.inertia.inverse([1., 0., 0.]), [0.; 3]);
}
#[test]
fn torque_free_rotation_rigidity_energy_momentum_and_reproducibility() {
    fn run(dt: f64) -> (f64, [f64; 4]) {
        let mut s = Simulation::native(100., dt).unwrap();
        let id = s
            .world
            .spawn_molecule(Arc::new(template(0).unwrap()), [50.; 3], [0., 0., 0., 1.])
            .unwrap();
        s.world
            .set_molecule_angular_velocity(id, [0.017, 0.023, -0.011])
            .unwrap();
        let l = s.world.molecules()[0].angular_momentum;
        let e = s.world.molecules()[0].rotational_energy();
        let lengths: Vec<_> = s
            .world
            .bonds()
            .iter()
            .map(|b| {
                norm(sub(
                    s.world.atoms()[b.a].position,
                    s.world.atoms()[b.b].position,
                ))
            })
            .collect();
        s.step((1000. / dt) as u32).unwrap();
        let m = &s.world.molecules()[0];
        assert_eq!(m.angular_momentum, l);
        assert!(norm(sub(s.world.atoms()[0].position, [50., 50., 50.])) > 0.01);
        for (b, d) in s.world.bonds().iter().zip(lengths) {
            assert!(
                (norm(sub(
                    s.world.atoms()[b.a].position,
                    s.world.atoms()[b.b].position
                )) - d)
                    .abs()
                    < 1e-12
            );
        }
        assert!((m.rotation.iter().map(|x| x * x).sum::<f64>() - 1.).abs() < 1e-14);
        ((m.rotational_energy() - e).abs() / e, m.rotation)
    }
    let a = run(0.1);
    assert!(a.0 < 1e-4, "{}", a.0);
    assert_eq!(a, run(0.1));
    let fine = run(0.05).0;
    assert!(fine < a.0 * 0.4, "coarse={} fine={fine}", a.0);
}
#[test]
fn rotational_equipartition_and_zero_temperature() {
    let mut s = Simulation::native(1000., 0.1).unwrap();
    s.world
        .batch_spawn_molecules(Arc::new(template(0).unwrap()), 2000, 10, 100)
        .unwrap();
    s.world.apply_thermal_velocities(300., 42).unwrap();
    let o = Observables::rigid(&s.world);
    assert_eq!(o.degrees_of_freedom, 12000);
    assert!(
        (o.temperature_kelvin / 300. - 1.).abs() < 0.04,
        "{}",
        o.temperature_kelvin
    );
    let rot = s
        .world
        .molecules()
        .iter()
        .map(|m| m.rotational_energy())
        .sum::<f64>();
    let expected = 2000. * 1.5 * mol_engine_core::actions::KB_VELOCITY * 300.;
    assert!((rot / expected - 1.).abs() < 0.04);
    s.world.apply_thermal_velocities(0., 42).unwrap();
    assert_eq!(Observables::rigid(&s.world).kinetic_kcal_mol, 0.);
}
#[test]
fn topology_preserves_rotating_survivor() {
    let mut s = Simulation::native(100., 0.2).unwrap();
    let d = Arc::new(template(1).unwrap());
    let a = s
        .world
        .spawn_molecule(d.clone(), [20.; 3], [0., 0., 0., 1.])
        .unwrap();
    s.world.apply_thermal_velocities(300., 42).unwrap();
    s.step(100).unwrap();
    let before = s.world.atoms().to_vec();
    let q = s.world.molecules()[0].rotation;
    let b = s
        .world
        .spawn_molecule(d, [80.; 3], [0., 0., 0., 1.])
        .unwrap();
    s.world.remove_molecule(b).unwrap();
    assert_eq!(s.world.molecules()[0].id, a);
    assert_eq!(s.world.molecules()[0].rotation, q);
    for (x, y) in before.iter().zip(s.world.atoms()) {
        assert_eq!(x.position, y.position);
        assert_eq!(x.velocity, y.velocity);
    }
}

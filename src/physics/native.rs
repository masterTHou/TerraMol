use super::PhysicsBackend;
use crate::{
    error::{Error, Result},
    geometry::*,
    world::World,
};
use std::collections::BTreeMap;
/// Rigid translations + elastic bounding-sphere impulses. No atomistic forces/torques.
pub struct NativeBackend;
impl PhysicsBackend for NativeBackend {
    fn initialize(&mut self, _: &World, _: f64) -> Result<()> {
        Ok(())
    }
    fn sync_topology(&mut self, _: &World) -> Result<()> {
        Ok(())
    }
    fn sync_state(&mut self, _: &World) -> Result<()> {
        Ok(())
    }
    fn step(&mut self, w: &mut World, dt: f64, steps: u32) -> Result<()> {
        if !dt.is_finite() || dt <= 0. || steps > 1_000_000 {
            return Err(Error::invalid("invalid fixed timestep or step count"));
        }
        let min_radius = w
            .molecules
            .iter()
            .map(|m| m.definition.radius)
            .fold(f64::INFINITY, f64::min);
        for _ in 0..steps {
            // Adaptive internal substeps limit travel to 1/4 smallest sphere radius.
            let speed = w
                .molecules
                .iter()
                .map(|m| norm(m.velocity))
                .fold(0., f64::max);
            let subdivisions = (speed * dt / (min_radius * 0.25)).ceil().max(1.);
            if subdivisions > 10000. {
                return Err(Error::invalid("native collision substep budget exceeded"));
            }
            let n = subdivisions as u32;
            for _ in 0..n {
                for m in &mut w.molecules {
                    m.center = add(m.center, scale(m.velocity, dt / n as f64));
                    reflect(m, w.sim_box.size);
                }
                let cell = w
                    .molecules
                    .iter()
                    .map(|m| 2. * m.definition.radius)
                    .fold(1., f64::max);
                let mut grid: BTreeMap<[i64; 3], Vec<usize>> = BTreeMap::new();
                for (i, m) in w.molecules.iter().enumerate() {
                    grid.entry(m.center.map(|v| (v / cell).floor() as i64))
                        .or_default()
                        .push(i);
                }
                let mut pairs = Vec::new();
                for (i, m) in w.molecules.iter().enumerate() {
                    let key = m.center.map(|v| (v / cell).floor() as i64);
                    for x in -1..=1 {
                        for y in -1..=1 {
                            for z in -1..=1 {
                                if let Some(js) = grid.get(&[key[0] + x, key[1] + y, key[2] + z]) {
                                    for &j in js {
                                        if j > i {
                                            pairs.push((i, j));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                pairs.sort_unstable();
                for (i, j) in pairs {
                    let (left, right) = w.molecules.split_at_mut(j);
                    let a = &mut left[i];
                    let b = &mut right[0];
                    let d = sub(b.center, a.center);
                    let distance = norm(d);
                    let target = a.definition.radius + b.definition.radius;
                    if distance < target {
                        let normal = if distance > 1e-12 {
                            scale(d, 1. / distance)
                        } else {
                            [1., 0., 0.]
                        };
                        let inv_a = 1. / a.definition.mass;
                        let inv_b = 1. / b.definition.mass;
                        let closing = dot(sub(b.velocity, a.velocity), normal);
                        if closing < 0. {
                            let impulse = -2. * closing / (inv_a + inv_b);
                            a.velocity = sub(a.velocity, scale(normal, impulse * inv_a));
                            b.velocity = add(b.velocity, scale(normal, impulse * inv_b));
                        }
                        let correction = (target - distance) / (inv_a + inv_b);
                        a.center = sub(a.center, scale(normal, correction * inv_a));
                        b.center = add(b.center, scale(normal, correction * inv_b));
                    }
                }
                for m in &mut w.molecules {
                    reflect(m, w.sim_box.size);
                }
            }
            w.refresh_atoms();
        }
        Ok(())
    }
}
fn reflect(m: &mut crate::molecule::MoleculeInstance, size: f64) {
    let r = m.definition.radius;
    let width = size - 2. * r;
    for k in 0..3 {
        let x = m.center[k] - r;
        let folded = x.rem_euclid(2. * width);
        if folded > width {
            m.center[k] = r + 2. * width - folded;
            m.velocity[k] = -m.velocity[k];
        } else {
            m.center[k] = r + folded;
        }
        if m.center[k] <= r && m.velocity[k] < 0. || m.center[k] >= size - r && m.velocity[k] > 0. {
            m.velocity[k] = -m.velocity[k];
        }
    }
}

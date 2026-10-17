//! Scientific observables. No COM momentum subtraction; no thermostat.
use crate::{actions::KB_VELOCITY, geometry::dot, world::World};
pub const VELOCITY_ENERGY: f64 = 2390.05736153349; // amu (A/fs)^2 to kcal/mol
#[derive(Clone, Copy, Debug)]
pub struct Observables {
    pub kinetic_kcal_mol: f64,
    pub potential_kcal_mol: Option<f64>,
    pub total_kcal_mol: Option<f64>,
    pub temperature_kelvin: f64,
    pub degrees_of_freedom: u32,
}
impl Observables {
    pub fn rigid(w: &World) -> Self {
        let k = w
            .molecules()
            .iter()
            .map(|m| {
                0.5 * m.definition.mass() * dot(m.velocity, m.velocity) + m.rotational_energy()
            })
            .sum::<f64>();
        let dof = w
            .molecules()
            .iter()
            .map(|m| 3 + m.definition.inertia.dof())
            .sum();
        Self::new(k * VELOCITY_ENERGY, None, dof)
    }
    pub fn atomic(w: &World, potential: f64) -> Self {
        let k = w
            .atoms()
            .iter()
            .map(|a| 0.5 * a.mass * dot(a.velocity, a.velocity))
            .sum::<f64>();
        Self::new(
            k * VELOCITY_ENERGY,
            Some(potential),
            3 * w.atoms().len() as u32,
        )
    }
    fn new(k: f64, p: Option<f64>, dof: u32) -> Self {
        Self {
            kinetic_kcal_mol: k,
            potential_kcal_mol: p,
            total_kcal_mol: p.map(|p| p + k),
            temperature_kelvin: if dof == 0 {
                0.
            } else {
                2. * k / (dof as f64 * KB_VELOCITY * VELOCITY_ENERGY)
            },
            degrees_of_freedom: dof,
        }
    }
}

use crate::{
    error::{Code, Error, Result},
    geometry::*,
    molecule::MoleculeId,
    world::World,
};
/// k_B in amu (Angstrom/fs)^2 / K.
pub const KB_VELOCITY: f64 = 8.31446261815324e-7;
impl World {
    pub fn set_molecule_velocity(&mut self, id: MoleculeId, velocity: Vec3) -> Result<()> {
        if !finite(velocity) {
            return Err(Error::invalid("invalid velocity"));
        }
        let m = self
            .molecules
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| Error::new(Code::NotFound, "molecule not found"))?;
        let delta = sub(velocity, m.velocity);
        m.velocity = velocity;
        for a in &mut self.atoms[m.atom_start..m.atom_start + m.definition.atoms.len()] {
            a.velocity = add(a.velocity, delta);
        }
        self.state_version += 1;
        Ok(())
    }
    /// Independent Maxwell COM samples; no removal of net momentum or exact rescaling.
    pub fn apply_thermal_velocities(&mut self, temperature: f64, seed: u64) -> Result<()> {
        if !temperature.is_finite() || !(0.0..=1e8).contains(&temperature) {
            return Err(Error::invalid("temperature must be 0..1e8 K"));
        }
        let mut rng = Rng::new(seed);
        let values: Vec<_> = self
            .molecules
            .iter()
            .map(|m| {
                (
                    m.id,
                    std::array::from_fn(|_| {
                        rng.normal() * (KB_VELOCITY * temperature / m.definition.mass).sqrt()
                    }),
                )
            })
            .collect();
        for (id, v) in values {
            self.set_molecule_velocity(id, v)?;
        }
        Ok(())
    }
    pub fn apply_directed_flow(&mut self, delta: Vec3) -> Result<()> {
        if !finite(delta) {
            return Err(Error::invalid("invalid flow"));
        }
        let values: Vec<_> = self
            .molecules
            .iter()
            .map(|m| (m.id, add(m.velocity, delta)))
            .collect();
        if values.iter().any(|(_, v)| !finite(*v)) {
            return Err(Error::invalid("velocity overflow"));
        }
        for (id, v) in values {
            self.set_molecule_velocity(id, v)?;
        }
        Ok(())
    }
}

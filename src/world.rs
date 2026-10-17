use crate::{
    error::{Code, Error, Result},
    geometry::*,
    molecule::*,
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct AtomState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub number: u32,
    pub molecule_id: MoleculeId,
    pub mass: f64,
}
pub struct World {
    pub(crate) molecules: Vec<MoleculeInstance>,
    pub(crate) atoms: Vec<AtomState>,
    pub(crate) bonds: Vec<Bond>,
    pub(crate) sim_box: SimBox,
    pub(crate) topology_version: u64,
    pub(crate) state_version: u64,
    pub(crate) next_id: u64,
    identity: u64,
}
impl World {
    pub fn new(size: f64) -> Result<Self> {
        static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let sim_box = SimBox::new(size)?;
        use std::sync::atomic::Ordering::Relaxed;
        let identity = loop {
            let current = ID.load(Relaxed);
            let next = current
                .checked_add(1)
                .ok_or_else(|| Error::new(Code::Capacity, "world identities exhausted"))?;
            if ID
                .compare_exchange_weak(current, next, Relaxed, Relaxed)
                .is_ok()
            {
                break current;
            }
        };
        Ok(Self {
            molecules: Vec::new(),
            atoms: Vec::new(),
            bonds: Vec::new(),
            sim_box,
            topology_version: 0,
            state_version: 0,
            next_id: 1,
            identity,
        })
    }
    pub fn sim_box(&self) -> SimBox {
        self.sim_box
    }
    pub fn topology_version(&self) -> u64 {
        self.topology_version
    }
    pub fn state_version(&self) -> u64 {
        self.state_version
    }
    pub(crate) fn identity(&self) -> u64 {
        self.identity
    }
    pub fn molecules(&self) -> &[MoleculeInstance] {
        &self.molecules
    }
    pub fn atoms(&self) -> &[AtomState] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
    pub fn spawn_molecule(
        &mut self,
        definition: Arc<MoleculeDefinition>,
        center: Vec3,
        rotation: [f64; 4],
    ) -> Result<MoleculeId> {
        let id = self.insert_molecule(definition, center, rotation)?;
        self.rebuild();
        Ok(id)
    }
    fn insert_molecule(
        &mut self,
        definition: Arc<MoleculeDefinition>,
        center: Vec3,
        rotation: [f64; 4],
    ) -> Result<MoleculeId> {
        if self.atoms.len().saturating_add(definition.atoms.len()) > 1_000_000 {
            return Err(Error::new(Code::Capacity, "v1 atom budget is 1000000"));
        }
        let qn = rotation.iter().map(|x| x * x).sum::<f64>();
        if !qn.is_finite()
            || (qn - 1.).abs() > 1e-8
            || definition.radius * 2. >= self.sim_box.size
            || !self.sim_box.contains(center, definition.radius)
        {
            return Err(Error::invalid(
                "rotation must be normalized; molecular sphere must fit box",
            ));
        }
        if self
            .molecules
            .iter()
            .any(|m| norm(sub(m.center, center)) < m.definition.radius + definition.radius)
        {
            return Err(Error::new(
                Code::Capacity,
                "molecular bounding spheres overlap",
            ));
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| Error::new(Code::Capacity, "ID exhausted"))?;
        self.molecules.push(MoleculeInstance {
            id,
            definition,
            center,
            velocity: [0.; 3],
            rotation,
            atom_start: 0,
        });
        Ok(id)
    }
    /// Atomic batch: an unsuccessful placement rolls back the entire batch (IDs stay retired).
    pub fn batch_spawn_molecules(
        &mut self,
        definition: Arc<MoleculeDefinition>,
        count: u32,
        seed: u64,
        max_retries: u32,
    ) -> Result<Vec<MoleculeId>> {
        if count > 100_000 || max_retries == 0 || max_retries > 1_000_000 {
            return Err(Error::invalid("invalid batch limits"));
        }
        if self
            .atoms
            .len()
            .saturating_add(definition.atoms.len().saturating_mul(count as usize))
            > 1_000_000
        {
            return Err(Error::new(Code::Capacity, "batch exceeds v1 atom budget"));
        }
        if count > 0 && definition.radius * 2. >= self.sim_box.size {
            return Err(Error::invalid("molecule does not fit box"));
        }
        let start = self.molecules.len();
        let mut rng = Rng::new(seed);
        let mut ids = Vec::new();
        for _ in 0..count {
            let mut placed = false;
            for _ in 0..max_retries {
                let p = std::array::from_fn(|_| {
                    definition.radius + rng.uniform() * (self.sim_box.size - 2. * definition.radius)
                });
                if let Ok(id) = self.insert_molecule(definition.clone(), p, rng.rotation()) {
                    ids.push(id);
                    placed = true;
                    break;
                }
            }
            if !placed {
                self.molecules.truncate(start);
                self.rebuild();
                return Err(Error::new(
                    Code::Capacity,
                    "placement retries exhausted; batch rolled back",
                ));
            }
        }
        if count > 0 {
            self.rebuild();
        }
        Ok(ids)
    }
    pub fn remove_molecule(&mut self, id: MoleculeId) -> Result<()> {
        let i = self
            .molecules
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| Error::new(Code::NotFound, "molecule not found"))?;
        self.molecules.remove(i);
        self.rebuild();
        Ok(())
    }
    pub fn clear_all(&mut self) {
        self.molecules.clear();
        self.rebuild();
    }
    pub(crate) fn rebuild(&mut self) {
        self.atoms.clear();
        self.bonds.clear();
        for m in &mut self.molecules {
            m.atom_start = self.atoms.len();
            for a in &m.definition.atoms {
                self.atoms.push(AtomState {
                    position: add(m.center, rotate(m.rotation, a.local)),
                    velocity: m.velocity,
                    number: a.element.number,
                    molecule_id: m.id,
                    mass: a.element.mass,
                });
            }
            self.bonds.extend(m.definition.bonds.iter().map(|b| Bond {
                a: b.a + m.atom_start,
                b: b.b + m.atom_start,
                order: b.order,
            }));
        }
        self.topology_version += 1;
        self.state_version += 1;
    }
    pub(crate) fn refresh_atoms(&mut self) {
        for m in &self.molecules {
            for (i, a) in m.definition.atoms.iter().enumerate() {
                let s = &mut self.atoms[m.atom_start + i];
                s.position = add(m.center, rotate(m.rotation, a.local));
                s.velocity = m.velocity;
            }
        }
        self.state_version += 1;
    }
    #[allow(dead_code)] // Used by the optional atomistic backend in Phase 4.
    pub(crate) fn refresh_centers(&mut self) {
        for m in &mut self.molecules {
            let slice = &self.atoms[m.atom_start..m.atom_start + m.definition.atoms.len()];
            m.center = scale(
                slice
                    .iter()
                    .fold([0.; 3], |v, a| add(v, scale(a.position, a.mass))),
                1. / m.definition.mass,
            );
            m.velocity = scale(
                slice
                    .iter()
                    .fold([0.; 3], |v, a| add(v, scale(a.velocity, a.mass))),
                1. / m.definition.mass,
            );
        }
        self.state_version += 1;
    }
}

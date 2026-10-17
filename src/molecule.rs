use crate::{
    chemistry::Element,
    error::{Error, Result},
    geometry::*,
};
use std::sync::Arc;
pub type MoleculeId = u64;
#[derive(Debug, Clone)]
pub struct AtomDefinition {
    pub element: Element,
    pub local: Vec3,
}
#[derive(Debug, Clone, Copy)]
pub struct Bond {
    pub a: usize,
    pub b: usize,
    pub order: Option<u8>,
}
#[derive(Debug)]
pub struct MoleculeDefinition {
    pub(crate) name: String,
    pub(crate) atoms: Vec<AtomDefinition>,
    pub(crate) bonds: Vec<Bond>,
    pub(crate) mass: f64,
    pub(crate) source_com: Vec3,
    pub(crate) radius: f64,
    pub inertia: crate::physics::rigid::Inertia,
}
impl MoleculeDefinition {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn atoms(&self) -> &[AtomDefinition] {
        &self.atoms
    }
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
    pub fn mass(&self) -> f64 {
        self.mass
    }
    pub fn source_com(&self) -> Vec3 {
        self.source_com
    }
    pub fn radius(&self) -> f64 {
        self.radius
    }
    pub fn new(name: &str, mut atoms: Vec<AtomDefinition>, bonds: Vec<Bond>) -> Result<Self> {
        if atoms.is_empty()
            || atoms.len() > 100_000
            || atoms.iter().any(|a| {
                !finite(a.local)
                    || !a.element.mass.is_finite()
                    || a.element.mass <= 0.
                    || !a.element.vdw.is_finite()
                    || a.element.vdw <= 0.
            })
            || bonds
                .iter()
                .any(|b| b.a >= atoms.len() || b.b >= atoms.len() || b.a == b.b)
        {
            return Err(Error::invalid("invalid molecular definition"));
        }
        let mass: f64 = atoms.iter().map(|a| a.element.mass).sum();
        let com = scale(
            atoms
                .iter()
                .fold([0.; 3], |p, a| add(p, scale(a.local, a.element.mass))),
            1. / mass,
        );
        if !mass.is_finite() || !finite(com) {
            return Err(Error::invalid("invalid molecular mass/center"));
        }
        for a in &mut atoms {
            a.local = sub(a.local, com);
        }
        let radius = atoms
            .iter()
            .map(|a| norm(a.local) + a.element.vdw)
            .fold(0., f64::max);
        if !radius.is_finite() || radius <= 0. || radius > 1e12 {
            return Err(Error::invalid("invalid molecular radius"));
        }
        let inertia_trace = atoms
            .iter()
            .map(|a| a.element.mass * dot(a.local, a.local))
            .sum::<f64>();
        if !inertia_trace.is_finite() {
            return Err(Error::invalid("inertia tensor overflow"));
        }
        let inertia = crate::physics::rigid::Inertia::from_atoms(&atoms);
        Ok(Self {
            inertia,
            name: name.into(),
            atoms,
            bonds,
            mass,
            source_com: com,
            radius,
        })
    }
}
#[derive(Clone)]
pub struct MoleculeInstance {
    pub id: MoleculeId,
    pub definition: Arc<MoleculeDefinition>,
    pub center: Vec3,
    pub velocity: Vec3,
    pub rotation: [f64; 4],
    /// World-frame angular momentum, amu Angstrom^2/fs.
    pub angular_momentum: Vec3,
    pub atom_start: usize,
}

impl MoleculeInstance {
    pub fn angular_velocity(&self) -> Vec3 {
        rotate(
            self.rotation,
            self.definition
                .inertia
                .inverse(rotate(conjugate(self.rotation), self.angular_momentum)),
        )
    }
    pub fn rotational_energy(&self) -> f64 {
        0.5 * dot(self.angular_momentum, self.angular_velocity())
    }
}

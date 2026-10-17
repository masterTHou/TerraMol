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
    pub name: String,
    pub atoms: Vec<AtomDefinition>,
    pub bonds: Vec<Bond>,
    pub mass: f64,
    pub source_com: Vec3,
    pub radius: f64,
}
impl MoleculeDefinition {
    pub fn new(name: &str, mut atoms: Vec<AtomDefinition>, bonds: Vec<Bond>) -> Result<Self> {
        if atoms.is_empty()
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
        let mass = atoms.iter().map(|a| a.element.mass).sum();
        let com = scale(
            atoms
                .iter()
                .fold([0.; 3], |p, a| add(p, scale(a.local, a.element.mass))),
            1. / mass,
        );
        for a in &mut atoms {
            a.local = sub(a.local, com);
        }
        let radius = atoms
            .iter()
            .map(|a| norm(a.local) + a.element.vdw)
            .fold(0., f64::max);
        Ok(Self {
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
    pub atom_start: usize,
}

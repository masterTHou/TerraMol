//! Contracts for future proposal generators. No search/optimizer is implemented in v1.
pub mod dihedral;
use crate::{
    error::{Error, Result},
    geometry::{finite, Vec3},
    molecule::MoleculeDefinition,
};
use std::collections::BTreeMap;
pub struct ConformerRequest {
    pub seed: u64,
    pub max_conformers: u32,
    pub options: BTreeMap<String, String>,
}
pub struct EnergyEstimate {
    pub kcal_per_mol: f64,
    pub method: String,
    pub parameterization: Option<String>,
}
pub struct Conformer {
    pub coordinates: Vec<Vec3>,
    pub energy: Option<EnergyEstimate>,
    pub metadata: BTreeMap<String, String>,
}
impl Conformer {
    pub fn new(
        definition: &MoleculeDefinition,
        coordinates: Vec<Vec3>,
        energy: Option<EnergyEstimate>,
        metadata: BTreeMap<String, String>,
    ) -> Result<Self> {
        if coordinates.len() != definition.atoms.len()
            || coordinates.iter().any(|v| !finite(*v))
            || energy
                .as_ref()
                .is_some_and(|e| !e.kcal_per_mol.is_finite() || e.method.trim().is_empty())
        {
            return Err(Error::invalid(
                "invalid conformer coordinates or energy provenance",
            ));
        }
        Ok(Self {
            coordinates,
            energy,
            metadata,
        })
    }
}
pub trait ConformerGenerator: Send {
    fn generate(
        &mut self,
        definition: &MoleculeDefinition,
        request: &ConformerRequest,
    ) -> Result<Vec<Conformer>>;
}

//! Backend-independent headless actions/observations. No reward or trainer is implied.
use crate::{
    actions::Action,
    error::Result,
    geometry::Vec3,
    molecule::{MoleculeDefinition, MoleculeId},
    snapshot::{Snapshot, ABI_VERSION},
    world::World,
    Simulation,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct SpawnBatch {
    pub definition: Arc<MoleculeDefinition>,
    pub count: u32,
    pub seed: u64,
    pub max_retries: u32,
}
#[derive(Clone)]
pub struct ResetSpec {
    pub batches: Vec<SpawnBatch>,
    pub thermal_kelvin: f64,
    pub thermal_seed: u64,
}
pub struct MoleculeObservation {
    pub id: MoleculeId,
    pub center: Vec3,
    pub velocity: Vec3,
    pub mass_amu: f64,
}
pub struct Observation {
    pub schema_version: u32,
    pub step_count: u64,
    pub time_fs: f64,
    pub snapshot: Snapshot,
    pub atoms: Vec<crate::world::AtomState>,
    pub molecules: Vec<MoleculeObservation>,
}
impl Simulation {
    /// Seeded geometry/velocities repeat; IDs remain retired across episodes.
    pub fn reset(&mut self, spec: ResetSpec) -> Result<()> {
        let mut staged = World::new(self.world.sim_box.size)?;
        staged.next_id = self.world.next_id;
        for b in spec.batches {
            staged.batch_spawn_molecules(b.definition, b.count, b.seed, b.max_retries)?;
        }
        staged.apply_thermal_velocities(spec.thermal_kelvin, spec.thermal_seed)?;
        staged.topology_version = self.world.topology_version + 1;
        staged.state_version = self.world.state_version + 1;
        // Failures invalidate sync caches; world replacement happens only after synchronization.
        self.topology = u64::MAX;
        self.state = u64::MAX;
        self.backend.sync_topology(&staged)?;
        self.backend.sync_state(&staged)?;
        self.world = staged;
        self.world_identity = self.world.identity();
        self.topology = self.world.topology_version;
        self.state = self.world.state_version;
        self.elapsed_steps = 0;
        Ok(())
    }
    pub fn observe(&self) -> Result<Observation> {
        Ok(Observation {
            schema_version: ABI_VERSION,
            step_count: self.elapsed_steps,
            time_fs: self.elapsed_steps as f64 * self.dt_fs,
            snapshot: Snapshot::capture(&self.world)?,
            atoms: self.world.atoms().to_vec(),
            molecules: self
                .world
                .molecules()
                .iter()
                .map(|m| MoleculeObservation {
                    id: m.id,
                    center: m.center,
                    velocity: m.velocity,
                    mass_amu: m.definition.mass,
                })
                .collect(),
        })
    }
    pub fn act(&mut self, action: Action) -> Result<Observation> {
        match action {
            Action::Spawn {
                definition,
                center,
                rotation,
            } => {
                self.world.spawn_molecule(definition, center, rotation)?;
            }
            Action::Batch(b) => {
                self.world
                    .batch_spawn_molecules(b.definition, b.count, b.seed, b.max_retries)?;
            }
            Action::Remove(id) => self.world.remove_molecule(id)?,
            Action::Clear => self.world.clear_all(),
            Action::Velocity { id, value } => self.world.set_molecule_velocity(id, value)?,
            Action::Thermal { kelvin, seed } => {
                self.world.apply_thermal_velocities(kelvin, seed)?
            }
            Action::Flow(v) => self.world.apply_directed_flow(v)?,
            Action::Step(n) => self.step(n)?,
            Action::Reset(spec) => self.reset(spec)?,
        }
        self.observe()
    }
}

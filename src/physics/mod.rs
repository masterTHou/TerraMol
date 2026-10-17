#[cfg(feature = "lammps")]
pub mod lammps_backend;
#[cfg(feature = "lammps")]
pub mod lammps_ffi;
pub mod native;
pub mod observables;
pub mod rigid;
use crate::{error::Result, world::World};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    Rigid,
    AtomisticLennardJones,
}
pub trait PhysicsBackend: Send {
    fn kind(&self) -> BackendKind;
    fn observables(&mut self, world: &World) -> Result<observables::Observables>;
    fn initialize(&mut self, world: &World, dt_fs: f64) -> Result<()>;
    fn sync_topology(&mut self, world: &World) -> Result<()>;
    fn sync_state(&mut self, world: &World) -> Result<()>;
    fn step(&mut self, world: &mut World, dt_fs: f64, steps: u32) -> Result<()>;
}

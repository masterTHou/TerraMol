pub mod native;
use crate::{error::Result, world::World};
pub trait PhysicsBackend: Send {
    fn initialize(&mut self, world: &World, dt_fs: f64) -> Result<()>;
    fn sync_topology(&mut self, world: &World) -> Result<()>;
    fn sync_state(&mut self, world: &World) -> Result<()>;
    fn step(&mut self, world: &mut World, dt_fs: f64, steps: u32) -> Result<()>;
}

pub mod actions;
pub mod chemistry;
pub mod error;
pub mod geometry;
pub mod molecule;
pub mod physics;
pub mod world;

use error::Result;
use physics::{native::NativeBackend, PhysicsBackend};
use world::World;

/// Owns the backend and the authoritative world. Clients never integrate atoms.
pub struct Simulation {
    pub world: World,
    backend: Box<dyn PhysicsBackend>,
    topology: u64,
    state: u64,
    pub dt_fs: f64,
}
impl Simulation {
    pub fn native(box_size: f64, dt_fs: f64) -> Result<Self> {
        Self::new(World::new(box_size)?, dt_fs, Box::new(NativeBackend))
    }
    pub fn new(world: World, dt_fs: f64, mut backend: Box<dyn PhysicsBackend>) -> Result<Self> {
        if !dt_fs.is_finite() || dt_fs <= 0.0 {
            return Err(error::Error::invalid(
                "timestep must be positive and finite",
            ));
        }
        backend.initialize(&world, dt_fs)?;
        Ok(Self {
            world,
            backend,
            topology: u64::MAX,
            state: u64::MAX,
            dt_fs,
        })
    }
    pub fn step(&mut self, steps: u32) -> Result<()> {
        if steps > 1_000_000 {
            return Err(error::Error::invalid("step limit is 1000000"));
        }
        if self.topology != self.world.topology_version {
            self.backend.sync_topology(&self.world)?;
            self.topology = self.world.topology_version;
        }
        if self.state != self.world.state_version {
            self.backend.sync_state(&self.world)?;
        }
        self.backend.step(&mut self.world, self.dt_fs, steps)?;
        self.state = self.world.state_version;
        Ok(())
    }
}

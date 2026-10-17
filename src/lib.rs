pub mod actions;
pub mod chemistry;
pub mod conformer;
pub mod environment;
pub mod error;
pub mod ffi;
pub mod geometry;
pub mod molecule;
pub mod physics;
pub mod snapshot;
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
    world_identity: u64,
    dt_fs: f64,
    elapsed_steps: u64,
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
            world_identity: 0,
            dt_fs,
            elapsed_steps: 0,
        })
    }
    pub fn step(&mut self, steps: u32) -> Result<()> {
        if steps > 1_000_000 {
            return Err(error::Error::invalid("step limit is 1000000"));
        }
        if self.world_identity != self.world.identity() {
            self.topology = u64::MAX;
            self.state = u64::MAX;
        }
        let next_step = self
            .elapsed_steps
            .checked_add(steps as u64)
            .ok_or_else(|| error::Error::new(error::Code::Capacity, "step counter exhausted"))?;
        if self.topology != self.world.topology_version {
            self.backend.sync_topology(&self.world)?;
            self.topology = self.world.topology_version;
        }
        if self.state != self.world.state_version {
            self.backend.sync_state(&self.world)?;
        }
        self.backend.step(&mut self.world, self.dt_fs, steps)?;
        self.state = self.world.state_version;
        self.world_identity = self.world.identity();
        self.elapsed_steps = next_step;
        Ok(())
    }
    pub fn dt_fs(&self) -> f64 {
        self.dt_fs
    }
    pub fn elapsed_steps(&self) -> u64 {
        self.elapsed_steps
    }
}

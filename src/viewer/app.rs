use crate::{
    chemistry::{template, ELEMENTS},
    environment::{ResetSpec, SpawnBatch},
    error::{Code, Error, Result},
    geometry::Vec3,
    snapshot::Snapshot,
    Simulation,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

pub const BOX_SIZE: f64 = 40.0;
const STEPS_PER_SECOND: f64 = 1000.0;

#[derive(Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Playback { running: bool },
    Speed { value: f64 },
    Spawn { kind: u32, count: u32 },
    Reset { kind: u32, seed: u64, kelvin: f64 },
    Thermal { kelvin: f64 },
    Velocity { id: String, value: Vec3 },
    Remove { id: String },
    Clear {},
    Shutdown {},
}

#[derive(Serialize)]
pub struct AtomDto {
    position: [f32; 3],
    velocity: [f32; 3],
    element: u32,
    molecule_id: String,
}
#[derive(Serialize)]
pub struct BondDto {
    a: u32,
    b: u32,
    order: u32,
}
#[derive(Serialize)]
pub struct MoleculeDto {
    id: String,
    name: String,
    center: Vec3,
    velocity: Vec3,
    mass_amu: f64,
}
#[derive(Serialize)]
pub struct ElementDto {
    number: u32,
    symbol: &'static str,
    cpk: [f32; 3],
    vdw: f64,
}
#[derive(Serialize)]
pub struct State {
    pub revision: String,
    pub topology_version: String,
    pub state_version: String,
    pub running: bool,
    pub speed: f64,
    pub temperature_setting: f64,
    pub box_size: f64,
    pub timestep_fs: f64,
    pub steps: String,
    pub time_fs: f64,
    pub error: Option<String>,
    pub atoms: Vec<AtomDto>,
    pub bonds: Vec<BondDto>,
    pub molecules: Vec<MoleculeDto>,
    pub elements: Vec<ElementDto>,
}

pub struct App {
    pub simulation: Simulation,
    running: bool,
    speed: f64,
    temperature: f64,
    revision: u64,
    seed: u64,
    debt: f64,
    error: Option<String>,
}
impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            simulation: Simulation::native(BOX_SIZE, 1.)?,
            running: true,
            speed: 1.,
            temperature: 300.,
            revision: 0,
            seed: 42,
            debt: 0.,
            error: None,
        };
        app.reset(0, 42, 300.)?;
        Ok(app)
    }
    fn reset(&mut self, kind: u32, seed: u64, kelvin: f64) -> Result<()> {
        temperature(kelvin)?;
        self.simulation.reset(ResetSpec {
            batches: vec![SpawnBatch {
                definition: Arc::new(template(kind)?),
                count: 50,
                seed,
                max_retries: 1000,
            }],
            thermal_kelvin: kelvin,
            thermal_seed: seed,
        })?;
        self.seed = seed;
        self.temperature = kelvin;
        self.debt = 0.;
        Ok(())
    }
    pub fn apply(&mut self, action: Action) -> Result<bool> {
        let mut shutdown = false;
        match action {
            Action::Playback { running } => {
                self.running = running;
                self.debt = 0.;
            }
            Action::Speed { value } => {
                if !value.is_finite() || !(0.1..=10.).contains(&value) {
                    return Err(Error::invalid("播放速度必须在 0.1 到 10 倍之间"));
                }
                self.speed = value;
            }
            Action::Spawn { kind, count } => {
                if ![1, 50].contains(&count) {
                    return Err(Error::invalid("每次只能生成 1 或 50 个分子"));
                }
                if self.simulation.world.molecules().len() + count as usize > 1000 {
                    return Err(Error::new(Code::Capacity, "可视化场景最多支持 1000 个分子"));
                }
                let definition = Arc::new(template(kind)?);
                self.seed = self.seed.wrapping_add(1);
                let ids = self
                    .simulation
                    .world
                    .batch_spawn_molecules(definition, count, self.seed, 1000)?;
                self.simulation.world.apply_thermal_velocities_for(
                    &ids,
                    self.temperature,
                    self.seed,
                )?;
            }
            Action::Reset { kind, seed, kelvin } => {
                self.reset(kind, seed, kelvin)?;
                self.running = true;
            }
            Action::Thermal { kelvin } => {
                temperature(kelvin)?;
                self.seed = self.seed.wrapping_add(1);
                self.simulation
                    .world
                    .apply_thermal_velocities(kelvin, self.seed)?;
                self.temperature = kelvin;
            }
            Action::Velocity { id, value } => self
                .simulation
                .world
                .set_molecule_velocity(parse_id(&id)?, value)?,
            Action::Remove { id } => self.simulation.world.remove_molecule(parse_id(&id)?)?,
            Action::Clear {} => self.simulation.world.clear_all(),
            Action::Shutdown {} => {
                shutdown = true;
                self.running = false;
            }
        }
        self.error = None;
        self.revision += 1;
        Ok(shutdown)
    }
    /// Wall time schedules an integer number of fixed 1 fs Rust steps, never an integration dt.
    pub fn tick(&mut self, elapsed: Duration) {
        if !self.running {
            return;
        }
        self.debt =
            (self.debt + elapsed.as_secs_f64().min(0.1) * STEPS_PER_SECOND * self.speed).min(200.);
        let steps = self.debt.floor().min(100.) as u32;
        if steps == 0 {
            return;
        }
        match self.simulation.step(steps) {
            Ok(()) => self.debt -= steps as f64,
            Err(e) => {
                self.error = Some(format!("模拟已暂停：{}", e.message));
                self.running = false;
                self.debt = 0.;
            }
        }
        self.revision += 1;
    }
    pub fn state(&self) -> Result<State> {
        let snap = Snapshot::capture(&self.simulation.world)?;
        Ok(State {
            revision: self.revision.to_string(),
            topology_version: snap.header.topology_version.to_string(),
            state_version: snap.header.state_version.to_string(),
            running: self.running,
            speed: self.speed,
            temperature_setting: self.temperature,
            box_size: BOX_SIZE,
            timestep_fs: self.simulation.dt_fs(),
            steps: self.simulation.elapsed_steps().to_string(),
            time_fs: self.simulation.elapsed_steps() as f64 * self.simulation.dt_fs(),
            error: self.error.clone(),
            atoms: snap
                .atoms
                .iter()
                .map(|a| AtomDto {
                    position: a.position,
                    velocity: a.velocity,
                    element: a.element,
                    molecule_id: a.molecule_id.to_string(),
                })
                .collect(),
            bonds: snap
                .bonds
                .iter()
                .map(|b| BondDto {
                    a: b.a,
                    b: b.b,
                    order: b.order,
                })
                .collect(),
            molecules: self
                .simulation
                .world
                .molecules()
                .iter()
                .map(|m| MoleculeDto {
                    id: m.id.to_string(),
                    name: m.definition.name().to_owned(),
                    center: m.center,
                    velocity: m.velocity,
                    mass_amu: m.definition.mass(),
                })
                .collect(),
            elements: ELEMENTS
                .iter()
                .map(|e| ElementDto {
                    number: e.number,
                    symbol: e.symbol,
                    cpk: e.cpk,
                    vdw: e.vdw,
                })
                .collect(),
        })
    }
}
fn parse_id(id: &str) -> Result<u64> {
    id.parse().map_err(|_| Error::invalid("无效的分子编号"))
}
fn temperature(t: f64) -> Result<()> {
    if !t.is_finite() || !(0.0..=5000.).contains(&t) {
        Err(Error::invalid("热速度温度必须在 0 到 5000 K 之间"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_and_fixed_step_pause() {
        let mut app = App::new().unwrap();
        assert_eq!(app.state().unwrap().atoms.len(), 150);
        app.tick(Duration::from_millis(20));
        assert_eq!(app.simulation.elapsed_steps(), 20);
        app.apply(Action::Playback { running: false }).unwrap();
        app.tick(Duration::from_secs(1));
        assert_eq!(app.simulation.elapsed_steps(), 20);
    }
    #[test]
    fn actions_and_reset_ids() {
        let mut app = App::new().unwrap();
        let old = app.simulation.world.molecules()[0].id;
        app.apply(Action::Reset {
            kind: 1,
            seed: 42,
            kelvin: 300.,
        })
        .unwrap();
        assert_eq!(app.state().unwrap().atoms.len(), 250);
        assert!(app.simulation.world.molecules().iter().all(|m| m.id != old));
        app.apply(Action::Spawn { kind: 2, count: 1 }).unwrap();
        let id = app
            .simulation
            .world
            .molecules()
            .last()
            .unwrap()
            .id
            .to_string();
        app.apply(Action::Velocity {
            id: id.clone(),
            value: [0.01, 0., 0.],
        })
        .unwrap();
        app.apply(Action::Remove { id }).unwrap();
        assert_eq!(app.simulation.world.molecules().len(), 50);
        assert!(app.apply(Action::Speed { value: f64::NAN }).is_err());
    }
}

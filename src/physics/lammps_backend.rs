use super::{lammps_ffi::Lammps, PhysicsBackend};
use crate::{
    error::{Code, Error, Result},
    geometry::finite,
    world::World,
};
use std::path::{Path, PathBuf};
/// Explicit single-species LJ sandbox. Bonded molecules require future parameterization.
#[derive(Clone, Copy)]
pub struct LennardJones {
    pub epsilon_kcal_mol: f64,
    pub sigma_angstrom: f64,
    pub cutoff_angstrom: f64,
}
impl LennardJones {
    fn validate(self) -> Result<()> {
        if [
            self.epsilon_kcal_mol,
            self.sigma_angstrom,
            self.cutoff_angstrom,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0.)
        {
            return Err(Error::invalid("LJ parameters must be finite and positive"));
        }
        Ok(())
    }
}
pub struct LammpsBackend {
    path: PathBuf,
    engine: Option<Lammps>,
    parameters: LennardJones,
    dt: f64,
    id_to_index: Vec<usize>,
}
impl LammpsBackend {
    pub fn new(path: impl AsRef<Path>, parameters: LennardJones) -> Result<Self> {
        parameters.validate()?;
        Ok(Self {
            path: path.as_ref().to_owned(),
            engine: None,
            parameters,
            dt: 1.,
            id_to_index: Vec::new(),
        })
    }
    pub fn total_energy(&mut self) -> Result<f64> {
        self.engine
            .as_mut()
            .ok_or_else(|| Error::new(Code::Backend, "backend not initialized"))?
            .energy()
    }
    fn rebuild(&mut self, w: &World) -> Result<()> {
        if w.molecules()
            .iter()
            .any(|m| m.definition.atoms.len() != 1 || !m.definition.bonds.is_empty())
            || w.atoms().first().is_some_and(|first| {
                w.atoms()
                    .iter()
                    .any(|a| a.number != first.number || a.mass != first.mass)
            })
        {
            return Err(Error::new(Code::Unsupported,"LJ backend accepts only a single species of monatomic particles with identical masses; water/methane/benzene need a bonded force field"));
        }
        if w.atoms().len() > i32::MAX as usize {
            return Err(Error::new(Code::Capacity, "LAMMPS ID range exceeded"));
        }
        let mut l = Lammps::open(&self.path)?;
        let p = self.parameters;
        for command in ["units real".to_owned(),"atom_style atomic".into(),"boundary f f f".into(),"atom_modify map array sort 0 0.0".into(),format!("region box block 0 {} 0 {} 0 {} units box",w.sim_box.size,w.sim_box.size,w.sim_box.size),"create_box 1 box".into(),format!("mass 1 {}",w.atoms().first().map(|a|a.mass).unwrap_or(1.)),format!("pair_style lj/cut {}",p.cutoff_angstrom),format!("pair_coeff 1 1 {} {} {}",p.epsilon_kcal_mol,p.sigma_angstrom,p.cutoff_angstrom),"pair_modify shift yes".into(),"neighbor 2.0 bin".into(),"neigh_modify every 1 delay 0 check yes".into(),format!("timestep {}",self.dt),"fix integrate all nve".into(),"fix walls all wall/reflect xlo EDGE xhi EDGE ylo EDGE yhi EDGE zlo EDGE zhi EDGE units box".into(),"thermo_style custom step temp pe ke etotal".into(),"compute_modify thermo_temp extra/dof 0".into(),"thermo_modify norm no lost error".into()] {l.command(&command)?;}
        let ids: Vec<_> = (1..=w.atoms().len() as i32).collect();
        let x: Vec<_> = w.atoms().iter().flat_map(|a| a.position).collect();
        let v: Vec<_> = w.atoms().iter().flat_map(|a| a.velocity).collect();
        if !ids.is_empty() {
            l.create_atoms(&ids, &x, &v)?;
            l.command("run 0")?;
        }
        self.id_to_index = (0..w.atoms().len()).collect();
        self.engine = Some(l);
        Ok(())
    }
}
impl PhysicsBackend for LammpsBackend {
    fn kind(&self) -> super::BackendKind {
        super::BackendKind::AtomisticLennardJones
    }
    fn observables(&mut self, w: &World) -> Result<super::observables::Observables> {
        if w.atoms().is_empty() {
            return Ok(super::observables::Observables::atomic(w, 0.));
        }
        let l = self
            .engine
            .as_mut()
            .ok_or_else(|| Error::new(Code::Backend, "uninitialized LAMMPS"))?;
        l.command("run 0 post no")?;
        let p = l.thermo_value("pe")?;
        let k = l.thermo_value("ke")?;
        let t = l.thermo_value("temp")?;
        if ![k, p, t].iter().all(|v| v.is_finite()) {
            return Err(Error::new(Code::Backend, "nonfinite LAMMPS observable"));
        }
        Ok(super::observables::Observables {
            kinetic_kcal_mol: k,
            potential_kcal_mol: Some(p),
            total_kcal_mol: Some(k + p),
            temperature_kelvin: t,
            degrees_of_freedom: 3 * w.atoms().len() as u32,
        })
    }

    fn initialize(&mut self, w: &World, dt: f64) -> Result<()> {
        self.dt = dt;
        self.rebuild(w)
    }
    fn sync_topology(&mut self, w: &World) -> Result<()> {
        self.rebuild(w)
    }
    fn sync_state(&mut self, w: &World) -> Result<()> {
        if w.atoms().len() != self.id_to_index.len() {
            return Err(Error::new(
                Code::Backend,
                "topology must be synchronized first",
            ));
        }
        if w.atoms().is_empty() {
            return Ok(());
        }
        let l = self
            .engine
            .as_mut()
            .ok_or_else(|| Error::new(Code::Backend, "uninitialized LAMMPS"))?;
        let mut x: Vec<_> = self
            .id_to_index
            .iter()
            .flat_map(|&i| w.atoms()[i].position)
            .collect();
        let mut v: Vec<_> = self
            .id_to_index
            .iter()
            .flat_map(|&i| w.atoms()[i].velocity)
            .collect();
        l.scatter("x", &mut x)?;
        l.scatter("v", &mut v)?;
        Ok(())
    }
    fn step(&mut self, w: &mut World, _dt: f64, steps: u32) -> Result<()> {
        if w.atoms().is_empty() || steps == 0 {
            return Ok(());
        }
        let l = self
            .engine
            .as_mut()
            .ok_or_else(|| Error::new(Code::Backend, "uninitialized LAMMPS"))?;
        l.command(&format!("run {steps}"))?;
        let x = l.gather("x", w.atoms().len())?;
        let v = l.gather("v", w.atoms().len())?;
        if x.as_chunks::<3>()
            .0
            .iter()
            .chain(v.as_chunks::<3>().0.iter())
            .any(|p| !finite(*p))
        {
            return Err(Error::new(Code::Backend, "nonfinite LAMMPS state"));
        }
        for (id, &i) in self.id_to_index.iter().enumerate() {
            w.atoms[i].position = x[id * 3..id * 3 + 3].try_into().unwrap();
            w.atoms[i].velocity = v[id * 3..id * 3 + 3].try_into().unwrap();
        }
        w.refresh_centers();
        Ok(())
    }
}

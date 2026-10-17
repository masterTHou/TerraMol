//! # Safety
//! Every non-null pointer must address writable/readable memory of the declared
//! length and remain valid throughout the call. No API retains caller pointers.
//! Handles are opaque monotonic tokens, never addresses. Calls are serialized.
#![allow(clippy::missing_safety_doc)]
use crate::{
    chemistry,
    error::{Code, Error, Result},
    snapshot::{Snapshot, ABI_VERSION},
    Simulation,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, Mutex, OnceLock},
};
struct Registry {
    next: u64,
    worlds: BTreeMap<u64, Simulation>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
thread_local! {static LAST_ERROR:RefCell<String>=const{RefCell::new(String::new())};}
fn boundary(f: impl FnOnce() -> Result<()>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => {
            LAST_ERROR.with(|s| s.borrow_mut().clear());
            Code::Ok as i32
        }
        Ok(Err(e)) => {
            LAST_ERROR.with(|s| *s.borrow_mut() = e.message);
            e.code as i32
        }
        Err(_) => {
            LAST_ERROR.with(|s| *s.borrow_mut() = "Rust panic contained at ABI boundary".into());
            Code::Panic as i32
        }
    }
}
fn registry() -> Result<std::sync::MutexGuard<'static, Registry>> {
    REGISTRY
        .get_or_init(|| {
            Mutex::new(Registry {
                next: 1,
                worlds: BTreeMap::new(),
            })
        })
        .lock()
        .map_err(|_| Error::new(Code::Panic, "registry poisoned; restart process"))
}
fn with_world(f: u64, op: impl FnOnce(&mut Simulation) -> Result<()>) -> Result<()> {
    op(registry()?
        .worlds
        .get_mut(&f)
        .ok_or_else(|| Error::new(Code::NotFound, "invalid or released handle"))?)
}
fn definition(kind: u32) -> Result<Arc<crate::molecule::MoleculeDefinition>> {
    static T: OnceLock<[Arc<crate::molecule::MoleculeDefinition>; 3]> = OnceLock::new();
    T.get_or_init(|| {
        std::array::from_fn(|i| {
            Arc::new(chemistry::template(i as u32).expect("validated template"))
        })
    })
    .get(kind as usize)
    .cloned()
    .ok_or_else(|| Error::invalid("unknown template"))
}
fn pointer<T>(p: *const T) -> Result<()> {
    if p.is_null() || !(p as usize).is_multiple_of(std::mem::align_of::<T>()) {
        Err(Error::invalid("null or misaligned pointer"))
    } else {
        Ok(())
    }
}
#[no_mangle]
pub unsafe extern "C" fn me_create_lj(
    path: *const u8,
    length: u32,
    size: f64,
    dt: f64,
    epsilon: f64,
    sigma: f64,
    cutoff: f64,
    out: *mut u64,
) -> i32 {
    boundary(|| {
        pointer(out)?;
        unsafe { out.write(0) };
        pointer(path)?;
        if length == 0 || length > 32768 {
            return Err(Error::invalid("library path length must be 1..32768"));
        }
        let name =
            std::str::from_utf8(unsafe { std::slice::from_raw_parts(path, length as usize) })
                .map_err(|_| Error::invalid("path must be UTF-8"))?;
        #[cfg(feature = "lammps")]
        {
            use crate::physics::lammps_backend::{LammpsBackend, LennardJones};
            let backend = LammpsBackend::new(
                name,
                LennardJones {
                    epsilon_kcal_mol: epsilon,
                    sigma_angstrom: sigma,
                    cutoff_angstrom: cutoff,
                },
            )?;
            let sim = Simulation::new(crate::world::World::new(size)?, dt, Box::new(backend))?;
            let mut r = registry()?;
            let id = r.next;
            r.next = r
                .next
                .checked_add(1)
                .ok_or_else(|| Error::new(Code::Capacity, "handle space exhausted"))?;
            r.worlds.insert(id, sim);
            unsafe { out.write(id) };
            Ok(())
        }
        #[cfg(not(feature = "lammps"))]
        {
            let _ = (name, size, dt, epsilon, sigma, cutoff);
            Err(Error::new(
                Code::Unsupported,
                "build with --features lammps",
            ))
        }
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_spawn_particle(
    handle: u64,
    number: u32,
    x: f64,
    y: f64,
    z: f64,
    out: *mut u64,
) -> i32 {
    boundary(|| {
        pointer(out)?;
        unsafe { out.write(0) };
        let e = chemistry::by_number(number)?;
        let d = Arc::new(crate::molecule::MoleculeDefinition::new(
            "Monatomic particle",
            vec![crate::molecule::AtomDefinition {
                element: e,
                local: [0.; 3],
            }],
            vec![],
        )?);
        with_world(handle, |s| {
            let id = s.world.spawn_molecule(d, [x, y, z], [0., 0., 0., 1.])?;
            unsafe { out.write(id) };
            Ok(())
        })
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_abi_version(out: *mut u32) -> i32 {
    boundary(|| {
        pointer(out)?;
        unsafe { out.write(ABI_VERSION) };
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_create(size: f64, dt: f64, out: *mut u64) -> i32 {
    boundary(|| {
        pointer(out)?;
        unsafe { out.write(0) };
        let sim = Simulation::native(size, dt)?;
        let mut r = registry()?;
        let id = r.next;
        r.next = r
            .next
            .checked_add(1)
            .ok_or_else(|| Error::new(Code::Capacity, "handle space exhausted"))?;
        r.worlds.insert(id, sim);
        unsafe { out.write(id) };
        Ok(())
    })
}
#[no_mangle]
pub extern "C" fn me_destroy(handle: u64) -> i32 {
    boundary(|| {
        registry()?
            .worlds
            .remove(&handle)
            .ok_or_else(|| Error::new(Code::NotFound, "invalid or released handle"))?;
        Ok(())
    })
}
#[no_mangle]
pub extern "C" fn me_step(handle: u64, steps: u32) -> i32 {
    boundary(|| with_world(handle, |s| s.step(steps)))
}
#[no_mangle]
pub extern "C" fn me_reset_template(
    handle: u64,
    kind: u32,
    count: u32,
    seed: u64,
    kelvin: f64,
) -> i32 {
    boundary(|| {
        with_world(handle, |s| {
            s.reset(crate::environment::ResetSpec {
                batches: vec![crate::environment::SpawnBatch {
                    definition: definition(kind)?,
                    count,
                    seed,
                    max_retries: 1000,
                }],
                thermal_kelvin: kelvin,
                thermal_seed: seed,
            })
        })
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_spawn(
    handle: u64,
    kind: u32,
    x: f64,
    y: f64,
    z: f64,
    out: *mut u64,
) -> i32 {
    boundary(|| {
        pointer(out)?;
        unsafe { out.write(0) };
        with_world(handle, |s| {
            let id = s
                .world
                .spawn_molecule(definition(kind)?, [x, y, z], [0., 0., 0., 1.])?;
            unsafe { out.write(id) };
            Ok(())
        })
    })
}
#[no_mangle]
pub extern "C" fn me_batch_spawn(
    handle: u64,
    kind: u32,
    count: u32,
    seed: u64,
    retries: u32,
) -> i32 {
    boundary(|| {
        with_world(handle, |s| {
            s.world
                .batch_spawn_molecules(definition(kind)?, count, seed, retries)?;
            Ok(())
        })
    })
}
#[no_mangle]
pub extern "C" fn me_remove(handle: u64, id: u64) -> i32 {
    boundary(|| with_world(handle, |s| s.world.remove_molecule(id)))
}
#[no_mangle]
pub extern "C" fn me_clear(handle: u64) -> i32 {
    boundary(|| {
        with_world(handle, |s| {
            s.world.clear_all();
            Ok(())
        })
    })
}
#[no_mangle]
pub extern "C" fn me_set_velocity(handle: u64, id: u64, x: f64, y: f64, z: f64) -> i32 {
    boundary(|| with_world(handle, |s| s.world.set_molecule_velocity(id, [x, y, z])))
}
#[no_mangle]
pub extern "C" fn me_thermal(handle: u64, kelvin: f64, seed: u64) -> i32 {
    boundary(|| with_world(handle, |s| s.world.apply_thermal_velocities(kelvin, seed)))
}
#[no_mangle]
pub extern "C" fn me_flow(handle: u64, x: f64, y: f64, z: f64) -> i32 {
    boundary(|| with_world(handle, |s| s.world.apply_directed_flow([x, y, z])))
}
#[no_mangle]
pub unsafe extern "C" fn me_snapshot_size(handle: u64, out: *mut u64) -> i32 {
    boundary(|| {
        pointer(out)?;
        with_world(handle, |s| {
            unsafe { out.write(Snapshot::capture(&s.world)?.byte_len() as u64) };
            Ok(())
        })
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_snapshot_copy(
    handle: u64,
    buffer: *mut u8,
    capacity: u64,
    written: *mut u64,
) -> i32 {
    boundary(|| {
        pointer(written)?;
        unsafe { written.write(0) };
        with_world(handle, |s| {
            let snap = Snapshot::capture(&s.world)?;
            let n = snap.byte_len();
            unsafe { written.write(n as u64) };
            if capacity < n as u64 {
                return Err(Error::new(
                    Code::Capacity,
                    "snapshot buffer too small; retry using required size",
                ));
            }
            pointer(buffer)?;
            let b = snap.bytes();
            unsafe { std::ptr::copy_nonoverlapping(b.as_ptr(), buffer, n) };
            Ok(())
        })
    })
}
#[repr(C)]
pub struct ElementInfo {
    pub number: u32,
    pub cpk: [f32; 3],
    pub vdw: f64,
    pub mass: f64,
}
#[repr(C)]
pub struct MoleculeInfo {
    pub center: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
}
#[no_mangle]
pub unsafe extern "C" fn me_molecule_info(handle: u64, id: u64, out: *mut MoleculeInfo) -> i32 {
    boundary(|| {
        pointer(out)?;
        with_world(handle, |s| {
            let m = s
                .world
                .molecules()
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| Error::new(Code::NotFound, "molecule not found"))?;
            unsafe {
                out.write(MoleculeInfo {
                    center: m.center,
                    velocity: m.velocity,
                    mass: m.definition.mass,
                })
            };
            Ok(())
        })
    })
}
#[no_mangle]
pub unsafe extern "C" fn me_element(number: u32, out: *mut ElementInfo) -> i32 {
    boundary(|| {
        pointer(out)?;
        let e = chemistry::by_number(number)?;
        unsafe {
            out.write(ElementInfo {
                number: e.number,
                cpk: e.cpk,
                vdw: e.vdw,
                mass: e.mass,
            })
        };
        Ok(())
    })
}
/// Does not clear the error being read. `required` includes the terminating NUL.
#[no_mangle]
pub unsafe extern "C" fn me_last_error(buffer: *mut u8, capacity: u64, required: *mut u64) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        if pointer(required).is_err() {
            return Code::Invalid as i32;
        }
        LAST_ERROR.with(|s| {
            let s = s.borrow();
            let n = s.len() + 1;
            unsafe { required.write(n as u64) };
            if capacity < n as u64 {
                return Code::Capacity as i32;
            }
            if buffer.is_null() {
                return Code::Invalid as i32;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(s.as_ptr(), buffer, s.len());
                buffer.add(s.len()).write(0);
            }
            Code::Ok as i32
        })
    }))
    .unwrap_or(Code::Panic as i32)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catches_panics() {
        assert_eq!(
            boundary(|| panic!("intentional test panic")),
            Code::Panic as i32
        );
        let message = LAST_ERROR.with(|s| s.borrow().clone());
        assert!(message.contains("panic contained"));
    }
}
/// Additive ABI v1 function; omega uses world-frame radians/fs.
#[no_mangle]
pub unsafe extern "C" fn me_set_angular_velocity(h: u64, id: u64, x: f64, y: f64, z: f64) -> i32 {
    boundary(|| with_world(h, |s| s.world.set_molecule_angular_velocity(id, [x, y, z])))
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MeObservables {
    pub kinetic: f64,
    pub potential: f64,
    pub total: f64,
    pub temperature: f64,
    pub degrees_of_freedom: u32,
    /// 1 means potential and total are meaningful; 0 means unavailable.
    pub has_potential: u32,
}
#[no_mangle]
pub unsafe extern "C" fn me_observables(h: u64, out: *mut MeObservables) -> i32 {
    boundary(|| {
        pointer(out)?;
        with_world(h, |s| {
            let o = s.observables()?;
            unsafe {
                out.write(MeObservables {
                    kinetic: o.kinetic_kcal_mol,
                    potential: o.potential_kcal_mol.unwrap_or(0.),
                    total: o.total_kcal_mol.unwrap_or(0.),
                    temperature: o.temperature_kelvin,
                    degrees_of_freedom: o.degrees_of_freedom,
                    has_potential: u32::from(o.potential_kcal_mol.is_some()),
                });
            }
            Ok(())
        })
    })
}

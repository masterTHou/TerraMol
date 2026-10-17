//! Dynamic C API bindings checked against installed 29Aug2024 update3 library.h.
//! Only serial, 32-bit tagint, exception-enabled builds are accepted.
use crate::error::{Code, Error, Result};
use libloading::Library;
use std::sync::Mutex;
static C_API_LOCK: Mutex<()> = Mutex::new(());
use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    path::Path,
};
type Open = unsafe extern "C" fn(c_int, *mut *mut c_char, *mut *mut c_void) -> *mut c_void;
type Close = unsafe extern "C" fn(*mut c_void);
type Command = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_char;
type Config = unsafe extern "C" fn() -> c_int;
type Setting = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type HasError = unsafe extern "C" fn(*mut c_void) -> c_int;
type LastError = unsafe extern "C" fn(*mut c_void, *mut c_char, c_int) -> c_int;
type CreateAtoms = unsafe extern "C" fn(
    *mut c_void,
    c_int,
    *const c_int,
    *const c_int,
    *const f64,
    *const f64,
    *const c_int,
    c_int,
) -> c_int;
type Transfer = unsafe extern "C" fn(*mut c_void, *const c_char, c_int, c_int, *mut c_void);
type Thermo = unsafe extern "C" fn(*mut c_void, *const c_char) -> f64;
pub(crate) struct Lammps {
    handle: *mut c_void,
    close: Close,
    command: Command,
    has_error: HasError,
    last_error: LastError,
    create: CreateAtoms,
    gather: Transfer,
    scatter: Transfer,
    thermo: Thermo,
    _library: Library,
}
// Serial builds only, exclusive &mut methods, no borrowed pointers escape.
// PhysicsBackend is Send but not Sync; FFI serializes calls through its registry.
unsafe impl Send for Lammps {}
fn failure(s: impl Into<String>) -> Error {
    Error::new(Code::Backend, s)
}
impl Lammps {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            let library = Library::new(path).map_err(|e| failure(e.to_string()))?;
            macro_rules! sym {
                ($name:literal,$ty:ty) => {
                    *library
                        .get::<$ty>(concat!($name, "\0").as_bytes())
                        .map_err(|e| failure(e.to_string()))?
                };
            }
            let mpi = sym!("lammps_config_has_mpi_support", Config);
            let exceptions = sym!("lammps_config_has_exceptions", Config);
            if mpi() != 0 || exceptions() == 0 {
                return Err(failure("v1 requires serial LAMMPS with exception support"));
            }
            let open = sym!("lammps_open_no_mpi", Open);
            let close = sym!("lammps_close", Close);
            let command = sym!("lammps_command", Command);
            let has_error = sym!("lammps_has_error", HasError);
            let last_error = sym!("lammps_get_last_error_message", LastError);
            let create = sym!("lammps_create_atoms", CreateAtoms);
            let gather = sym!("lammps_gather_atoms", Transfer);
            let scatter = sym!("lammps_scatter_atoms", Transfer);
            let thermo = sym!("lammps_get_thermo", Thermo);
            let setting = sym!("lammps_extract_setting", Setting);
            let args: Vec<_> = ["terramol", "-screen", "none", "-log", "none", "-nocite"]
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect();
            let mut argv: Vec<_> = args.iter().map(|s| s.as_ptr() as *mut c_char).collect();
            let handle = open(argv.len() as c_int, argv.as_mut_ptr(), std::ptr::null_mut());
            if handle.is_null() {
                return Err(failure("LAMMPS initialization returned null"));
            }
            let s = Self {
                handle,
                close,
                command,
                has_error,
                last_error,
                create,
                gather,
                scatter,
                thermo,
                _library: library,
            };
            let tags = setting(handle, c"tagint".as_ptr());
            let status = s.check();
            drop(_guard); // Drop may take the same global lock on an error path.
            if tags != 4 {
                return Err(failure("BIGBIG/64-bit atom IDs are unsupported"));
            }
            status?;
            Ok(s)
        }
    }
    fn check(&self) -> Result<()> {
        unsafe {
            if (self.has_error)(self.handle) != 0 {
                let mut b = [0 as c_char; 4096];
                (self.last_error)(self.handle, b.as_mut_ptr(), b.len() as c_int);
                return Err(failure(
                    CStr::from_ptr(b.as_ptr()).to_string_lossy().into_owned(),
                ));
            }
            Ok(())
        }
    }
    pub(crate) fn command(&mut self, text: &str) -> Result<()> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let c = CString::new(text).map_err(|_| failure("NUL in command"))?;
        unsafe {
            (self.command)(self.handle, c.as_ptr());
        }
        self.check()
    }
    // lammps_command's return is borrowed by LAMMPS: never free it.
    pub(crate) fn create_atoms(&mut self, ids: &[i32], x: &[f64], v: &[f64]) -> Result<()> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        if ids.len() > i32::MAX as usize || x.len() != ids.len() * 3 || v.len() != x.len() {
            return Err(failure("invalid creation lengths"));
        }
        let types = vec![1; ids.len()];
        let n = unsafe {
            (self.create)(
                self.handle,
                ids.len() as i32,
                ids.as_ptr(),
                types.as_ptr(),
                x.as_ptr(),
                v.as_ptr(),
                std::ptr::null(),
                0,
            )
        };
        self.check()?;
        if n != ids.len() as i32 {
            return Err(failure("LAMMPS created an unexpected number of atoms"));
        }
        Ok(())
    }
    pub(crate) fn scatter(&mut self, name: &str, data: &mut [f64]) -> Result<()> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let c = CString::new(name).unwrap();
        unsafe {
            (self.scatter)(self.handle, c.as_ptr(), 1, 3, data.as_mut_ptr().cast());
        }
        self.check()
    }
    pub(crate) fn gather(&mut self, name: &str, n: usize) -> Result<Vec<f64>> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let mut data = vec![0.; n * 3];
        let c = CString::new(name).unwrap();
        unsafe {
            (self.gather)(self.handle, c.as_ptr(), 1, 3, data.as_mut_ptr().cast());
        }
        self.check()?;
        Ok(data)
    }
    // Gather/scatter buffers are caller-owned Rust allocations, not C allocations.
    // No allocating extract APIs are used; lammps_free is therefore not needed.
    pub(crate) fn energy(&mut self) -> Result<f64> {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let e = unsafe { (self.thermo)(self.handle, c"etotal".as_ptr()) };
        self.check()?;
        Ok(e)
    }
}
impl Drop for Lammps {
    fn drop(&mut self) {
        let _guard = C_API_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        unsafe { (self.close)(self.handle) }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires LAMMPS_LIBRARY"]
    fn converts_command_error() {
        let path = std::env::var("LAMMPS_LIBRARY").unwrap();
        let mut l = Lammps::open(Path::new(&path)).unwrap();
        assert!(l.command("this_is_not_a_lammps_command").is_err());
    }
}

# Delivery log

Each phase is committed separately. The commit is the complete source delivery;
`git show <tag>` exposes every new/modified file and its actual code.

## Phase 1 — Chemistry, world, native backend

Implemented: eight elements, three explicit templates, strict single-frame XYZ,
shared definitions, stable IDs, transactional random batches, world versions,
mass-aware thermal sampling, rigid motion, grid broad phase and elastic impulses.

Validation: `cargo test --test core`, `cargo run --example headless`.
Six integration tests cover templates, malformed XYZ, overlap, deletion, rigid
distances, reflection, reproducibility, thermal statistics and pair impulses.

Limitations: conservative vdW bounding spheres; no angular dynamics, no forces,
no continuous collision detection. Substeps reduce tunneling but simultaneous
dense contacts use a single impulse sweep and can leave residual overlap.
XYZ inference is O(N²), does not determine bond order/aromaticity.
The built-in geometries are idealized educational templates, not force fields.
Native initialization/synchronization is intentionally stateless: the world
itself owns centers and velocities.

## Phase 2 — C ABI and snapshots

Implemented: cdylib/rlib, mutex-protected opaque token registry, stale-handle
rejection, thread-local UTF-8 errors, panic containment, caller-owned buffers,
versioned little-endian snapshot wire format and element queries.

Executed successfully on Windows x64: `cargo test --lib --test ffi`,
`cargo build --release`, `python tests/dll_smoke.py target/release/mol_engine_core.dll`.
The DLL smoke test loads the actual DLL, creates 50 waters, verifies 150 atoms /
100 bonds, and tests repeated destruction. Rust verifies sizes and offsets.

Limitations: null/length checks cannot establish validity of arbitrary foreign
addresses. Callers must obey the header's memory contract. Panic containment
requires unwind builds and does not handle abort/OOM or C++ termination. A poisoned
registry fails closed. Calls across all handles currently serialize for safety.
No zero-copy API. Python ctypes test is also a usable headless entry point, not
a packaged RL environment.

## Phase 3 — Unity integration

Implemented: all five requested C# classes, editor scene setup, pooled atoms and
bonds, CPK from Rust, whole-molecule highlighting, Rust COM velocity arrows,
debug controls and fixed-step scheduler. MaterialPropertyBlock and two shared
materials avoid one material per atom. Re-enable resets cached versions.

Executed: `powershell -ExecutionPolicy Bypass -File scripts/test-bridge.ps1`:
actual C# P/Invoke layout/lifecycle/snapshot tests pass against release DLL.
`cargo build --release` passes. Unity Editor is unavailable: full runtime/editor
validation remains pending; follow `unity/README.md`. The test does not compile
UnityEngine-dependent files or claim Play mode verification.

Limitations: idealized sphere/cylinder rendering, no bond-order graphics,
no camera navigation, no GPU instancing, per-atom Marshal boxing; suitable for
prototype scenes. Scene generator runs in the editor. No player binary is supplied.

## Phase 4 — Optional LAMMPS

Implemented: dynamically loaded official C API, RAII instance lifecycle,
serial/exception/tag-width checks, explicit real-unit LJ configuration, fixed
dt NVE, reflected box, caller-owned scatter/gather buffers, ID mapping, topology
rebuild versus state scatter, captured error conversion and C ABI LJ creation.
All LAMMPS C calls additionally serialize through a global adapter lock.

Executed successfully on Windows x64, 29Aug2024 update3, serial shared library
built from official pinned source using MSVC 19.51 (Visual Studio 18, x64):

```powershell
$env:LAMMPS_LIBRARY = 'D:\TerraMol\artifacts\lammps-msvc\Release\liblammps.dll'
cargo run --features lammps --example lammps_lj
cargo test --features lammps --test lammps -- --ignored --test-threads=1
cargo test --features lammps --lib -- --ignored --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
```

Example energy: -0.011173996232 → -0.011173996231 kcal/mol after 1000 x 0.1 fs.
Physical/mapping/reproducibility test, C ABI LJ lifecycle test and command-error
test all pass. `tests/lammps_header_check.cpp` was compiled against installed
29Aug2024 update3 headers; `tests/c_layout.c` compiled successfully as C11.
Default tests and standalone C# bridge still pass after capacity/alignment fixes.

An initial MinGW compiler produced a 32-bit DLL and loading failed with x64 Rust.
The final tests above use the successful MSVC x64 build. The downloaded Windows
wheel lacked a shared DLL; external sources/builds remain ignored in artifacts/.

Limitations: monatomic single species only, reflecting nonperiodic box, no
bonded force fields, no MPI, no 64-bit tags, no allocator-returning extract APIs,
no backend checkpoint rollback. Polyatomic templates are explicitly rejected.
This validates the specified LJ system, not water/methane/benzene dynamics.
Linux/macOS LAMMPS are covered by a configured CI job, not yet claimed locally.

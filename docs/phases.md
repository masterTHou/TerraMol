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

## Phase 5 — Extension contracts

Implemented: ConformerGenerator trait, validated coordinate/energy metadata,
signed dihedral measurement and graph-cut editing (rings rejected), unified
Action/Observation, f64 headless atom observations, deterministic staged reset,
monotonic IDs across episodes, C ABI reset, standalone Python adapter and C# reset.
Definitions expose read-only accessors; versions/box/timestep cannot be edited
without core operations. World identity invalidates backend caches on Rust world
replacement. Public C ABI remains v1; new symbols are additive.

Validation: `cargo test --test extensions` covers reset/action invariants,
dihedral geometry/ring rejection and energy provenance. Final regression commands
and actual results are recorded below after execution. There is no implemented
conformer proposal search, optimization, reward function, Gym wrapper or trainer.

Limits: dihedral geometry does not validate sterics; Python/C observe f32 render
data while Rust observations also expose f64 atom state. Reset does not guarantee
rollback of arbitrary future physics backend failures. See `docs/extensions.md`.

## Final local regression — 2026-10-17

Windows x64, Rust/Cargo 1.95.0, .NET Framework C# compiler, Python 3.12.3:

| Executed command | Result |
|---|---|
| `cargo fmt --check` | pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| `cargo test` | 14 tests pass; no LAMMPS installation needed |
| `cargo test --all-features` | 15 pass; 3 environment-dependent tests ignored |
| Three opt-in LAMMPS tests (commands above) | all explicitly executed and pass |
| `cargo build --release` | pass; Windows x64 cdylib produced |
| `python tests/dll_smoke.py target/release/mol_engine_core.dll` | pass |
| `python tests/python_headless.py target/release/mol_engine_core.dll` | pass |
| `powershell -ExecutionPolicy Bypass -File scripts/test-bridge.ps1` | pass, including seeded reset |
| `cargo run --example headless` | pass; 50 molecules / 150 atoms |
| `cargo run --features lammps --example lammps_lj` | pass; energy results above |
| C11 public header layout and installed LAMMPS header static assertions | pass |

Unity Editor/Play mode/player/IL2CPP remain unverified. Cross-platform CI is
configured; consult its actual run status rather than extrapolating local results.
No LAMMPS binary or downloaded wheel is included in the release.

## Executed cross-platform CI

[Run 39030938630](https://github.com/masterTHou/TerraMol/actions/runs/39030938630)
completed successfully for runtime commit `0f60882`:

- Windows, Ubuntu and macOS: format, strict Clippy, feature tests, release build,
  actual shared-library ABI smoke and Python reset/actions pass.
- Windows: standalone C# bridge layout/lifecycle/reset passes.
- Ubuntu: pinned serial LAMMPS build, header signature assertions, both physical /
  FFI integration tests, error conversion test and LJ example pass.

CI used Rust stable 1.99.1; local validation used 1.95.0. Initial current-stable
lint failures were fixed without suppressing warnings. Later commits only add
release documentation/packaging. macOS LAMMPS and Unity Editor remain unverified.

## v1.1 — Portable viewer

Added an optional `viewer` feature, standalone Rust EXE, loopback-only HTTP
transport, embedded offline WebGL/Canvas client, one-click Chinese launcher,
beginner guide, static-CRT Windows build and portable release packaging.
Scientific state and fixed-step integration stay in the Rust core. The browser
does not implement chemistry, thermal sampling or dynamics.

Executed locally: default tests (15 pass), viewer unit tests, strict all-feature
Clippy, static Windows EXE build, real-process HTTP acceptance, idle-exit test,
WebGL/Canvas browser checks, whole-molecule picking, velocity/arrow, deletion and
UI shutdown. `dumpbin /DEPENDENTS TerraMol.exe` lists Windows system DLLs only.
The default browser opens from the `.cmd` launcher. ZIP CRC checks and SHA-256
are generated by `scripts/package-viewer.py`. Full test contract and limits are
in `docs/viewer.md`; CI results are reported separately once executed.

Current local totals: `cargo test` = 15 passed; `cargo test --all-features` =
19 passed / 3 ignored. All three LAMMPS opt-in tests were also explicitly executed
and passed after the thermal-selection change. No ignored test is counted as passed.

Final v1.1 [CI run 39032916141](https://github.com/masterTHou/TerraMol/actions/runs/39032916141)
is successful for runtime commit `e7d2d0e`: Windows/Linux/macOS core, actual viewer
process/HTTP/lifecycle, Windows static-CRT packaging and C# bridge, and Ubuntu
LAMMPS all pass. The extracted local Windows package also passes from a Unicode /
spaced folder and unrelated working directory. Final precise input fields use
`step=any` so sampled velocities and fractional Kelvin values remain editable.

## v1.2 — Rotation and scientific experiment loop

Added point-mass rigid rotation, principal-axis thermal samples, explicit backend
kind/observable contracts, topology-state preservation, viewer LJ mode, measured
science, action/initial-state export and render trajectory replay. See
[v1.2 specification and acceptance](v1.2.md) for files, commands, numerical
approximations, tests and known limits. Portable LAMMPS runtime is compiled from
pinned unchanged source with static CRT; binary/source/third-party notices are
packaged together in the release.

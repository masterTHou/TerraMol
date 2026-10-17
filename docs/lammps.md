# Optional LAMMPS backend

`cargo build --release --features lammps` compiles the adapter. It dynamically
loads an explicitly supplied library path; default Rust builds have no native
LAMMPS dependency. The v1 adapter deliberately supports only serial builds,
32-bit atom tags and exception support. MPI/multi-rank execution is not supported.

## A parameterized minimal system

The example is a generic, single-species Lennard–Jones pair. Carbon is used
only for its 12.011 amu mass/element display; these parameters are **not** a
validated carbon force field:

| Setting | Value |
|---|---:|
| Units | `real`: A, fs, amu, kcal/mol |
| epsilon | 0.1 kcal/mol |
| sigma | 3.4 A |
| cutoff | 8.5 A; shifted energy |
| Integrator | NVE velocity Verlet |
| timestep | 0.1 fs |
| Box | 40 A, fixed boundaries + reflecting walls |
| Initial separation | 6 A |

```powershell
$env:LAMMPS_LIBRARY = 'C:\absolute\path\liblammps.dll'
cargo run --features lammps --example lammps_lj
cargo test --features lammps --test lammps -- --ignored --test-threads=1
cargo test --features lammps --lib -- --ignored --test-threads=1
```

Linux/macOS use an absolute `.so`/`.dylib` path. The tests check nonzero attraction,
energy drift below 1e-7 kcal/mol for the specified trajectory, repeatability,
delete/rebuild ID mapping, velocity scatter, empty-system rebuilding and invalid
command error conversion. They are opt-in and ignored in ordinary `cargo test`.
Do not treat an ignored test as a pass.

**Executed verification:** Windows x64 serial library compiled with MSVC from
29Aug2024 update3; the example and all three opt-in tests passed. The example's
energy changed from -0.011173996232 to -0.011173996231 kcal/mol. See
[phase log](phases.md) for exact commands and environment. Linux/macOS execution
is not implied by this result.

## C-API and ownership audit

Bindings were checked against the installed wheel's
`lammps/include/lammps/library.h` for 29Aug2024 update3, and the corresponding
[official header](https://github.com/lammps/lammps/blob/stable_29Aug2024_update3/src/library.h).
See [official implementation](https://github.com/lammps/lammps/blob/stable_29Aug2024_update3/src/library.cpp)
and [scatter/gather documentation](https://docs.lammps.org/Library_scatter.html).

| API | Signature / ownership |
|---|---|
| `open_no_mpi` | `(int, char**, void**) -> void*`; pass null third arg, own returned instance |
| `close` | `(void*) -> void`; RAII Drop closes before unloading library |
| `command` | `(void*, const char*) -> char*`; returned text is borrowed; never freed |
| `create_atoms` | `(void*, int, const int*, const int*, const double*, const double*, const int*, int) -> int`; caller arrays, copy on call |
| `gather_atoms` / `scatter_atoms` | `(void*, const char*, int, int, void*) -> void`; type 1 doubles, count 3; caller owns buffers |
| `has_error` | `(void*) -> int` |
| `get_last_error_message` | `(void*, char*, int) -> int`; caller buffer, clears captured error |
| `extract_setting` | `(void*, const char*) -> int`; checks tagint size |
| `get_thermo` | `(void*, const char*) -> double`; value, no allocation |

No C-allocated extraction buffer is requested. Consequently there is no buffer
requiring `lammps_free`; adding allocating extraction APIs later must wrap their
documented ownership separately. `lammps_close` releases instance-owned allocations.
Commands and creation check captured errors. C++ aborts/OOM are beyond Rust's
panic boundary; the supported library must retain its C API exception handling.

Topology changes build a fresh instance and assign IDs 1..N, with an explicit
ID-to-Rust-index map. Gather returns ID order, independent of internal atom
sorting. State edits scatter positions/velocities without rebuilding topology.
The engine never issues commands to renumber/delete atoms outside rebuilding.

## Build a compatible library

Use the pinned source tag and the [official CMake build workflow](https://docs.lammps.org/Build_cmake.html):

```sh
git clone --depth 1 --branch stable_29Aug2024_update3 https://github.com/lammps/lammps.git
cmake -S lammps/cmake -B lammps-build -D BUILD_SHARED_LIBS=ON -D BUILD_MPI=OFF -D BUILD_OMP=OFF -D CMAKE_BUILD_TYPE=Release
cmake --build lammps-build --target lammps --parallel 4
```

Windows MinGW: add `-G "MinGW Makefiles"` and select your C/C++ compilers if needed.
Check `g++ -dumpmachine`: the compiler must match the Rust target architecture.
With Visual Studio, use an x64 generator (`-A x64`) and build `--config Release`;
the library is then in the `Release/` subfolder. Compiler paths containing "w64"
do not by themselves guarantee a 64-bit target.
Keep any compiler runtime DLLs in the loader search path. Do not bundle LAMMPS
into a release without reviewing its GPL distribution obligations. TerraMol's
release contains its own source and native DLL only.

Water, methane, benzene, XYZ-derived polyatomic graphs, charges, bonds, angles,
dihedrals and long-range electrostatics require a future validated parameterization
layer. This backend returns Unsupported for them; it never silently runs them
as an LJ atom cloud. A `me_create_lj` C ABI entry and `me_spawn_particle` use the
same actions/snapshot/step APIs as the Native world.

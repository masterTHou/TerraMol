# Bundled runtime notices

TerraMol source is MIT licensed. The optional LAMMPS runtime is separately
copyrighted by its authors and licensed under GNU GPL version 2 (see
[LAMMPS-LICENSE.txt](LAMMPS-LICENSE.txt)). The combined viewer distribution
including LAMMPS is distributed under GPL v2 terms; TerraMol's original MIT
permissions remain available for its own source. No proprietary force fields
or molecular parameter datasets are bundled.

The full Windows viewer archive includes `liblammps.dll`, built from the unchanged
`stable_29Aug2024_update3` source, serial x64 SMALLBIG, no MPI/OpenMP, no optional
packages, static MSVC CRT. The exact source needed to build this configuration
(`src/`, `lib/`, `cmake/`, license and README) is available alongside the binary
as **TerraMol-v1.2.0-corresponding-source.tar.gz**. This also includes TerraMol
source, Cargo.lock, tests, viewer assets, scripts and these notices. Source
archives are provided without access restrictions or additional charge.

Rebuild LAMMPS with CMake and Visual Studio C++ x64:

```powershell
cd TerraMol
powershell -ExecutionPolicy Bypass -File scripts/build-lammps-runtime.ps1 -Source ../lammps-stable_29Aug2024_update3
```

Run from the TerraMol source directory (adjust the script/source paths accordingly).
Then `scripts/build-viewer.ps1`, `python scripts/package-viewer.py --with-lammps`.
Rust crates retain their upstream licenses (serde, serde_json: MIT/Apache-2.0;
libloading: ISC; transitive crates: see Cargo.lock and upstream package licenses).

Upstream: https://github.com/lammps/lammps/tree/stable_29Aug2024_update3
Official project: https://www.lammps.org/

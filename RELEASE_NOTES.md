# TerraMol v1.2.0 — Molecules can rotate

Native 分子现在同时平动和自由转动。版本新增完整惯性张量、角动量／姿态积分、
质量与惯性加权热采样、角速度设置，以及可直接切换的 LAMMPS LJ 实验模式。

## For beginners

Download **TerraMol-Viewer-v1.2.0-windows-x64-LAMMPS.zip**, extract **all** files,
and double-click **启动TerraMol.cmd** or **TerraMol.exe**. No Unity, Rust, Python,
Node.js, VC++ redistributable or separate LAMMPS installation is needed. The default
browser displays rotating water molecules. Switch “物理模型” to LAMMPS for generic
LJ particles with explicit parameters. A smaller ZIP omits the optional runtime.

## Changes

- Torque-free point-mass rigid rotation, normalized quaternion midpoint integration,
  deterministic Maxwell translation and principal-axis angular-momentum sampling.
- Exact preservation of surviving per-atom coordinates and velocities on topology changes.
- Explicit backend capability/observable contracts and transactional viewer mode changes.
- KE, PE, total, measured temperature, DOF, absolute energy change; unavailable Native
  PE/total clearly marked. LJ values come from LAMMPS, with 3N temperature convention.
- f64 initial state, configuration, seed and step-tagged action export; up to 1000
  sampled trajectory frames (20 MiB recording budget), JSON download/import and scrubbing without reintegration.
- Additive ABI 1 angular and observables functions; Python and C# bindings updated,
  unchanged render snapshot layout. Atom velocity includes rotation, COM is separate.
- Full portable runtime with license notices and matching source archive (including
  Rust dependency sources). TerraMol source remains MIT; bundled LAMMPS is GPL v2,
  and the combined distribution follows GPL v2 terms.

## Verification

Executed locally on Windows: `cargo fmt --check`, strict all-feature Clippy,
21 default Rust tests, 26 all-feature tests (4 opt-in tests ignored by default).
All 4 opt-in LAMMPS tests separately pass against the bundled runtime, including
analytic two-particle force/potential and NVE timestep convergence. Actual DLL
Python/C# and C/Rust/C# layouts pass. Real executable HTTP lifecycle/actions,
LJ switching/rejection, no-runtime failure, export metadata and idle exit tested.
Browser verified WebGL, download/import of 89 frames and scrubbing to the final frame.

## Scientific limits

Native collisions remain frictionless smooth bounding spheres, producing no torque
or translation/rotation energy exchange. No atomistic molecular force field is
implemented for water, methane or benzene. LJ is a generic monatomic sandbox:
ε=0.1 kcal/mol, σ=3.4 Å, cutoff=8.5 Å, shifted potential, mass=12.011 amu,
dt=0.1 fs, reflected walls. It is not a validated carbon potential. Measured
T includes overall flow; initial temperature sampling is not a thermostat.
Exported trajectories are observations, not restart checkpoints. Unity Editor,
IL2CPP and macOS LAMMPS remain unverified. Full definitions: [v1.2 documentation](https://github.com/masterTHou/TerraMol/blob/v1.2.0/docs/v1.2.md).

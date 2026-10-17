# Conformer and headless extension contracts

`ConformerGenerator` is a Rust trait producing `Vec<Conformer>` from a validated
immutable definition and a request carrying seed, maximum count and explicit
options. There is **no generator implementation** in v1: this is the intended
extension boundary. Empty or fake energy values are not used to imply optimization.

`Conformer::new` checks count, finite coordinates and optional energy provenance.
`EnergyEstimate` records kcal/mol, method and optional parameterization metadata.
No energy evaluator or minimizer is supplied. Zero is only meaningful if returned
by a real evaluator and accompanied by the method used.

`conformer::dihedral::measure` gives signed radians; `set` rotates the graph
component on the c-side of b→c after cutting b–c. It checks consecutive explicit
bonds, indices, nondegenerate geometry and rejects cycles reconnecting the cut.
It preserves internal bond lengths, but does not detect steric clashes or choose
chemically rotatable bonds. Benzene ring deformation is therefore Unsupported.
Coordinates are returned as a new array; no shared definition is mutated.

`Action` unifies spawn, batch, remove, clear, COM velocity, thermal sampling,
flow, fixed steps and reset. `Simulation::act` returns `Observation`, including
schema version, step count, physical time, copied f64 atom/molecule state and
the current rendering snapshot. It is usable without graphics or a foreign
runtime. There is no task-specific reward, termination criterion or policy.

`ResetSpec` stages an entire world, samples placements/velocities from explicit
seeds, synchronizes the backend, and replaces the authoritative state on success.
Failed geometric/thermal staging leaves the old world untouched. Backend failures
invalidate synchronization caches; backend rollback is not a general contract.
Successful reset sets elapsed time to zero, increments versions and preserves
monotonic ID allocation. The same seeds repeat physical state, while old IDs are
retired. This protects a stale Unity selection or Python action across episodes.

C/Python callers use `me_reset_template` plus the existing action/snapshot APIs.
Python's current observation is the f32 render schema; use Rust's f64 observer
for scientific precision until a versioned scientific C observation is added.

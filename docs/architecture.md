# Architecture decisions

## 001 — Rust is authoritative

Definitions encode chemical identity and geometry. Instances reference definitions
through Arc and own stable, monotonically increasing molecule IDs. The world's
contiguous atom array is rebuilt on topology changes while preserving surviving
atom coordinates/velocities. Snapshot bond indices are
valid only for that snapshot's topology version. An ID is never an array offset.
Atom identity across topology changes is not yet a public stable-ID contract.

## 002 — Backends are explicit models

Simulation owns one PhysicsBackend and fixed dt. Changes in topology trigger
`sync_topology`; actions trigger `sync_state`; `step(N)` advances fixed steps.
Native owns no separate copy of world data. LAMMPS owns an internal engine mirror
and copies back after run. The viewer switches modes by constructing a fresh validated
simulation, retiring IDs and incrementing topology versions; it does not reinterpret
a bonded molecular scene as LJ particles.
Backend step failures may leave that backend advanced internally; a scientific
experiment should discard/reset the failed episode rather than assume rollback.

The native broad phase uses ordered cells and sorted pairs for deterministic
iteration. Internal substeps limit travel; this is a bounded-cost approximation,
not an exact hard-sphere event solver. Dense simultaneous contacts and walls can
retain residual penetration. No potential energy exists for native collisions.

## 003 — A copied, versioned boundary

Opaque u64 tokens avoid dereferencing stale foreign handles. A registry mutex
serializes operations; no Vec, String, Result, trait object or retained caller
pointer crosses the C ABI. Error text is UTF-8 and thread-local. Invalid arbitrary
addresses remain the caller's responsibility, as with any C interface.

The snapshot wire format is little-endian: 40-byte header, 40-byte atom records,
16-byte bond records. Reserved fields are zero. Struct sizes and key offsets are
tested in Rust and C#. Query/copy is race-safe: a growing world returns Capacity
and the required size; each successful copy is a complete snapshot under lock.
Consumers must use the copied header, not a prior query's counts.

## 004 — Scientific units never follow rendering time

Internal f64 values use A, fs, A/fs, amu, K and kcal/mol. Native temperature samples
use k_B = 8.31446261815324e-7 amu (A/fs)^2 / K. Unity converts to f32 at the render
boundary, with 1 A = 1 scene unit. Its wall-clock accumulator only chooses N;
integration never consumes Time.deltaTime directly. No implicit unit conversion
is applied when transferring `units real` positions and velocities to LAMMPS.

## 005 — Extensions expose provenance

ConformerGenerator proposes candidates; energy and optimization are separate
future capabilities. Dihedral edits operate on explicit coordinate arrays and
validated graph cuts; rings require a dedicated algorithm. Observation/action
types support headless experimentation without implying a reward function or
learning algorithm. Reset keeps deterministic geometry/velocity seeds while
retiring prior molecule IDs so stale selections cannot target a new molecule.

## 006 — Rigid rotation and measured science

Native conserves world angular momentum and integrates orientation using the
point-mass inertia pseudoinverse. Thermal sampling includes principal-axis
rotation. Smooth bounding-sphere impulses carry no torque. Per-atom snapshot
velocities include rotation; COM velocity is a separate molecule observation.
Backend observables declare their DOF and potential availability; Native does not
fabricate a PE. See [v1.2 scientific definitions](v1.2.md).

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

The remaining phases add C ABI, Unity, optional LAMMPS and extension abstractions.

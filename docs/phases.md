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

# Contributing to TerraMol

Start with a focused issue or PR that states the behavior, scientific assumptions,
units, and a reproducible example. Separate numerical behavior changes from viewer
changes. Never implement simulation state or force laws in the Unity client.

Before submitting, run:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

Scientific changes require invariant or reference-data tests, plus numerical
tolerances justified by the model. Include RNG seeds and explicit parameters.
An XYZ structure alone is not a force field. Distinguish validated physical
models, pedagogical models and interface-only extensions.

ABI changes require matching C header/C# updates, layout tests, lifecycle tests
and an ABI version decision. Existing v1 fields must never silently change size
or meaning. Keep errors recoverable and prevent unwinds across FFI.

Optional dependency tests must be opt-in and record the library version/build.
Mark unexecuted tests honestly. See `docs/lammps.md` and `unity/README.md` for
environment-specific verification. Do not commit generated binaries, downloaded
dependencies, tokens, large trajectories or Unity Library directories.

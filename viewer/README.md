# Offline browser client

These files are embedded into the `terramol_viewer` Rust executable at build time.
Opening index.html directly is not supported; it needs the local Rust HTTP API.
End users launch the portable EXE/Chinese launcher, with no web development setup.

`app.js` implements UI/network transport and stale-response rejection.
`renderer.js` draws Rust-provided snapshots using shared WebGL sphere/cylinder
meshes, with Canvas projection fallback. It handles camera/picking geometry only.
There is no molecular topology inference, integrator, temperature sampler or
force law in JavaScript. Do not add those to the renderer.

No external fonts, npm dependencies, remote scripts or CDN requests are used.
`?renderer=canvas` forces compatibility rendering for acceptance tests.
See `docs/viewer.md` for ownership, scheduling, lifecycle and validation details.

Version 1.2: native torque-free rotation, optional LAMMPS LJ model, scientific
observables, experiment export and sampled trajectory replay. See
[scientific and verification contract](../docs/v1.2.md).

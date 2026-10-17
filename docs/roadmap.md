# Roadmap

| Milestone | Scope | Acceptance boundary |
|---|---|---|
| v1.0 core prototype | Rust native rigid scene, C ABI, Unity source, optional monatomic LJ, extension contracts | See phase verification log |
| v1.1 portable viewer | Self-contained EXE, offline browser rendering, beginner launcher and controls | Executed HTTP, lifecycle and browser acceptance tests; see viewer docs |
| Next: parameterization | Explicit atom typing, charges, bonds/angles/dihedrals, force-field provenance | Compare reference energies/forces with an independent system |
| Planned: atomistic workflows | Constraint policies, thermostat/barostat, trajectory and checkpoint formats | Validate ensembles, convergence and restart reproducibility |
| Planned: conformer workflows | Rotatable bond discovery, proposal generators, energy evaluation and minimizers | Geometry validity and benchmark datasets |
| Planned: Python environment | Versioned observations/actions, rewards, vector environments, deterministic reset | Headless parity tests and episode reproducibility |
| Later | GPU rendering, asynchronous snapshots, HPC/MPI backends, distributed experiments | Measure before optimizing; preserve ownership contracts |

Rows marked Next/Planned/Later are directions, not implemented capabilities or delivery dates.
Open design questions: validated force-field source/licensing, periodic molecules
and unwrapped coordinates, atom-level stable identity for reactive systems,
backend rollback/checkpoint semantics, energy evaluator boundary, thread affinity
for MPI, and snapshot throughput for large scenes.

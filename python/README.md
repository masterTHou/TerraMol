# Headless entry point

No dependencies beyond Python 3. Use the same native library as Unity:

```python
from terramol import HeadlessSimulation
with HeadlessSimulation("../target/release/mol_engine_core.dll") as env:
    observation = env.reset(kind=0, count=50, seed=42, kelvin=300)
    molecule_id = observation.atoms[0].molecule_id
    env.velocity(molecule_id, (0.01, 0.0, 0.0))
    next_observation = env.step(100)  # fixed Rust steps
```

Run from `python/` or add that directory to PYTHONPATH. Linux/macOS use their
`.so`/`.dylib` library path. `reset` repeats coordinates and velocities while
retiring previous molecule IDs; compare physical state rather than snapshot bytes
or IDs across episodes. Rust's f64 Observation is available to Rust clients; the
current C/Python observer intentionally reads the same f32 render snapshot as
Unity. A future scientific observation schema should provide f64 explicitly.

This thin adapter provides reset/actions/observations and context-managed
lifecycle. It does not implement Gym spaces, rewards, training, vectorized
environments, NumPy arrays or a Python conformer search API. Use one wrapper per
thread or externally serialize compound operations; the native registry still
protects each individual call. Errors must be read on the same calling thread.

Verification: `python tests/python_headless.py target/release/mol_engine_core.dll`.

# Unity viewer

1. Build/copy the native plugin: `pwsh -File scripts/install-unity-plugin.ps1`.
2. Open `unity/` using Unity 2022.3 LTS or a compatible newer editor. Use the
   Built-in render pipeline initially; enable the legacy Input Manager or Both.
3. Use **TerraMol → Create prototype scene**, save, and press Play.
4. Spawn 50 molecules, apply 300 K, enable Run. Click an atom to select its
   molecule; its CPK color highlights and a COM velocity arrow appears.

The scene generator adds the five runtime components and a camera/light.
Native coordinates map 1 Angstrom to 1 Unity unit. Velocity arrows multiply
Angstrom/fs by 200 for visibility; this does not change physical state.
The controller uses wall time only to schedule a number of fixed Rust steps.
At most 200 are executed per render frame, with bounded backlog after stalls.
Changing timestep/box inspector values requires restarting Play mode.

Atoms/bonds grow reusable pools; topology versions govern rebinding, state
versions govern transform updates. CPK/mass/COM come from Rust. Bonds have no
physics colliders; sphere colliders serve picking only. Two shared materials
and MaterialPropertyBlock avoid per-atom material allocation. No Unity rigidbody
or chemical topology logic exists here. Snapshots are reused borrowed objects.

`OnDisable`, `OnDestroy` and `OnApplicationQuit` dispose the bridge idempotently;
its finalizer is a fallback. Do not reload/replace a loaded DLL while in Play.

## Validation status

The standalone C# test compiles the exact bridge and loads the actual Rust DLL:
`pwsh -File scripts/test-bridge.ps1`. It checks layout, molecule metadata,
snapshots, step, deletion, clear and repeated dispose. Unity Editor is not
installed in the implementation environment: scene rendering, picking,
domain-reload behavior, player builds, URP shader stripping and OnGUI are
**not editor-verified**. To verify: enter/exit Play repeatedly, trigger a script
recompile, clear/delete a selected molecule, spawn batches while paused/running,
and inspect Console for exceptions and Memory Profiler for material leaks.

Linux/macOS: build `libmol_engine_core.so`/`.dylib` on the target platform and
place in `Assets/Plugins/` with the correct platform/architecture import settings.
IL2CPP and mobile are not yet validated. Standalone builds must include the
required Standard/Lit shader; URP requires a compatible material/shader setup.

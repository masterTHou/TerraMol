# Portable viewer architecture and validation

The standalone `terramol_viewer` executable links the Rust rlib directly and embeds
all HTML/CSS/JS with `include_bytes!`. It needs neither Unity nor a separately
loaded core DLL. End users only need their existing browser; no Python/Node/npm,
package manager or CDN is involved. Windows release builds use a static CRT.

## Ownership and scheduling

Browser → bounded local HTTP requests → Rust App/Simulation → NativeBackend.
The viewer is enabled by the optional `viewer` Cargo feature (serde/serde_json);
the default core still has no third-party dependencies. It does not select LAMMPS
or imply bonded-force-field support. The existing Unity/C/Python interfaces remain.

Rust owns all coordinates, topology, molecule IDs, thermal sampling, element
metadata and integration. Rendering only projects supplied coordinates into camera
space, draws shared meshes and performs geometric picking. COM/velocity arrows use
Rust-provided molecule data. New batches sample thermal velocities only for new
IDs, preserving existing molecules' motion. Kelvin input is a velocity sampling
setting, not an instantaneous thermometer or thermostat.

A wall-clock accumulator schedules integer steps at 1000 physical steps/s times
playback speed. Every step is 1 fs. Scheduling caps burst size and backlog after a
stall; wall-clock performance does not alter the integrator's dt. Browser rendering
uses requestAnimationFrame and independently polls complete snapshots (~20 Hz).
Monotonic response revisions reject stale overlapping HTTP replies. Molecule IDs
and counters travel as decimal strings, avoiding JavaScript's 53-bit integer limit.

## Local transport and resource lifecycle

The listener binds only 127.0.0.1 at an OS-assigned port. Every launch gets a
128-bit OS-generated random URL token. Routes require the expected Host and, when
present, Origin. No CORS access is granted. CSP limits scripts/styles/resources to
the bundled page and prevents framing. Filesystem routes are not exposed.
Requests have bounded headers/bodies/timeouts; chunked or duplicate framing is
rejected. Four workers and a bounded queue keep connection costs finite. Accepted
sockets explicitly use blocking I/O in workers, including on Windows where they
can inherit nonblocking mode from the listener.

The page's Exit action shuts down the listener, joins workers and drops the world.
Closing all tabs leaves no API activity; the process exits after 180 s of inactivity.
There is no installed service or persistent startup registration. Refreshing the
same page preserves the running world. Multiple launches create independent worlds.

The URL token is not an Internet login and expires with that process. The optional
`--ready-file` is for automated local tests, not normal user operation.

## Build and automated checks

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-viewer.ps1
python tests/viewer_smoke.py TerraMol.exe
python scripts/package-viewer.py
```

The build uses `target/portable` so a running development preview does not lock
the output build directory. Close a currently running root `TerraMol.exe` before
overwriting that root copy. Windows packages contain only EXE, launcher, beginner
guide and license. SHA-256 sums and a ZIP CRC check accompany packaging.

Unix development: `cargo build --release --features viewer --bin terramol_viewer`,
then `python tests/viewer_smoke.py target/release/terramol_viewer`.

The real-process smoke test checks startup, bundled assets/query strings, initial
counts, pause invariance, invalid actions, Host/Origin/token checks, reset, spawn,
velocity, delete/reindex, concurrent snapshots, clear, shutdown and idle exit.
Unit tests check fixed-step scheduling, app actions and strict request framing.
A core test checks selective thermal updates and failure atomicity.

Executed Windows acceptance: static-CRT x64 EXE smoke passes; `dumpbin /DEPENDENTS`
lists only USER32, synchronization API set, bcrypt, KERNEL32, ntdll and WS2_32.
The one-click launcher opened the actual default Chrome browser. Browser checks
confirmed WebGL drawing, template reset to methane (250 atoms), pause, atom picking
(whole-molecule selection), mass/velocity display, speed update/arrow, deletion
(147 atoms / 49 waters), Canvas fallback and UI shutdown. Mobile and desktop
layouts were inspected. The README screenshot is actual program output.

## Limits

Native uses rigid translations and approximate bounding spheres, not atomistic
MD. There is no client-side force field, complete conformer search or RL training.
The viewer caps scenes at 1000 molecules, but dense placement can exhaust retries
well before that cap. WebGL shares sphere/cylinder mesh buffers but does not yet
use GPU instancing; Canvas is a compatibility projection with approximate depth
sorting. Throughput/FPS depend on hardware and viewport. Session persistence,
trajectory export, scientific f64 HTTP observations and macOS/Linux prebuilt
packages are future work. The portable file is not a Unity player build.

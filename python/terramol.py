"""Dependency-free headless C-ABI client. This is not an RL trainer or Gym environment."""
import ctypes as c
from dataclasses import dataclass
from pathlib import Path
import struct


@dataclass(frozen=True)
class Atom:
    position: tuple
    velocity: tuple
    element: int
    molecule_id: int


@dataclass(frozen=True)
class Observation:
    abi_version: int
    state_version: int
    topology_version: int
    atoms: tuple
    bonds: tuple


class HeadlessSimulation:
    def __init__(self, library, box_size=100.0, dt_fs=1.0):
        self.lib = c.CDLL(str(Path(library).resolve()))
        self.handle = c.c_uint64()
        signatures = {
            'me_abi_version': [c.POINTER(c.c_uint32)],
            'me_create': [c.c_double, c.c_double, c.POINTER(c.c_uint64)],
            'me_destroy': [c.c_uint64],
            'me_step': [c.c_uint64, c.c_uint32],
            'me_reset_template': [c.c_uint64, c.c_uint32, c.c_uint32, c.c_uint64, c.c_double],
            'me_set_velocity': [c.c_uint64, c.c_uint64, c.c_double, c.c_double, c.c_double],
            'me_flow': [c.c_uint64, c.c_double, c.c_double, c.c_double],
            'me_snapshot_size': [c.c_uint64, c.POINTER(c.c_uint64)],
            'me_snapshot_copy': [c.c_uint64, c.c_void_p, c.c_uint64, c.POINTER(c.c_uint64)],
            'me_last_error': [c.c_void_p, c.c_uint64, c.POINTER(c.c_uint64)],
        }
        for name, args in signatures.items():
            fn = getattr(self.lib, name)
            fn.argtypes, fn.restype = args, c.c_int32
        version = c.c_uint32()
        self._check(self.lib.me_abi_version(c.byref(version)))
        if version.value != 1:
            raise RuntimeError('Unsupported TerraMol ABI')
        self._check(self.lib.me_create(box_size, dt_fs, c.byref(self.handle)))

    def _check(self, code):
        if not code:
            return
        n = c.c_uint64()
        self.lib.me_last_error(None, 0, c.byref(n))
        buf = c.create_string_buffer(n.value)
        self.lib.me_last_error(buf, n, c.byref(n))
        raise RuntimeError(f'Rust status {code}: {buf.value.decode("utf-8")}')

    def _alive(self):
        if not self.handle.value:
            raise RuntimeError('Simulation is closed')

    @staticmethod
    def _uint(value, bits):
        if not isinstance(value, int) or not 0 <= value < 2 ** bits:
            raise ValueError(f'Expected uint{bits}')
        return value

    def reset(self, kind=0, count=50, seed=42, kelvin=300.0):
        self._alive()
        self._check(self.lib.me_reset_template(self.handle, self._uint(kind, 32),
                    self._uint(count, 32), self._uint(seed, 64), kelvin))
        return self.observe()

    def step(self, steps=1):
        self._alive()
        self._check(self.lib.me_step(self.handle, self._uint(steps, 32)))
        return self.observe()

    def velocity(self, molecule_id, value):
        self._alive()
        if len(value) != 3:
            raise ValueError('Velocity requires three components')
        self._check(self.lib.me_set_velocity(self.handle, self._uint(molecule_id, 64), *value))

    def flow(self, value):
        self._alive()
        if len(value) != 3:
            raise ValueError('Flow requires three components')
        self._check(self.lib.me_flow(self.handle, *value))

    def observe(self):
        self._alive()
        n = c.c_uint64()
        self._check(self.lib.me_snapshot_size(self.handle, c.byref(n)))
        for _ in range(3):
            buf = c.create_string_buffer(n.value)
            status = self.lib.me_snapshot_copy(self.handle, buf, n.value, c.byref(n))
            if status == 3:
                continue
            self._check(status)
            abi, header, na, nb, state, topology, sa, sb = struct.unpack_from('<IIIIQQII', buf)
            if (abi, header, sa, sb) != (1, 40, 40, 16) or n.value != 40 + na*40 + nb*16:
                raise RuntimeError('Snapshot layout mismatch')
            atoms = []
            for i in range(na):
                a = struct.unpack_from('<ffffffIIQ', buf, 40 + i*40)
                atoms.append(Atom(a[:3], a[3:6], a[6], a[8]))
            bonds = tuple(struct.unpack_from('<IIII', buf, 40 + na*40 + i*16)[:3] for i in range(nb))
            if any(a >= na or b >= na for a, b, _ in bonds):
                raise RuntimeError('Snapshot bond index mismatch')
            return Observation(abi, state, topology, tuple(atoms), bonds)
        raise RuntimeError('Snapshot changed repeatedly')

    def close(self):
        if self.handle.value:
            self._check(self.lib.me_destroy(self.handle))
            self.handle.value = 0

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()

    def __del__(self):
        try:
            if getattr(self, 'handle', None):
                self.close()
        except Exception:
            pass

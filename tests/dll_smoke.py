"""Actual shared-library ABI test: python tests/dll_smoke.py target/release/mol_engine_core.dll"""
import ctypes as c
import struct
import sys
lib = c.CDLL(sys.argv[1])
lib.me_create.argtypes = [c.c_double, c.c_double, c.POINTER(c.c_uint64)]
lib.me_destroy.argtypes = [c.c_uint64]
lib.me_batch_spawn.argtypes = [c.c_uint64, c.c_uint32, c.c_uint32, c.c_uint64, c.c_uint32]
lib.me_snapshot_size.argtypes = [c.c_uint64, c.POINTER(c.c_uint64)]
lib.me_snapshot_copy.argtypes = [c.c_uint64, c.c_void_p, c.c_uint64, c.POINTER(c.c_uint64)]
h, n = c.c_uint64(), c.c_uint64()
assert lib.me_create(100, 1, c.byref(h)) == 0
try:
    assert lib.me_batch_spawn(h, 0, 50, 42, 1000) == 0
    assert lib.me_snapshot_size(h, c.byref(n)) == 0
    buf = c.create_string_buffer(n.value)
    assert lib.me_snapshot_copy(h, buf, n, c.byref(n)) == 0
    header = struct.unpack_from('<IIIIQQII', buf)
    assert header[:4] == (1, 40, 150, 100)
    assert header[-2:] == (40, 16)
    print('DLL ABI: 50 water molecules / 150 atoms / 100 bonds OK')
finally:
    assert lib.me_destroy(h) == 0
assert lib.me_destroy(h) == 2

# C ABI v1 contract

Call convention: `extern "C"` / cdecl. All exported functions return int32 status:
0 success, 1 invalid, 2 not found, 3 capacity, 4 backend, 5 panic, 6 unsupported.
`me_abi_version` writes uint32 value 1. `include/mol_engine_core.h` is the public
declaration. Rust, C and the exact C# bridge have layout tests.

## Lifecycle and errors

`me_create` produces a nonzero opaque uint64 handle. It is a monotonic registry
token, not an address. `me_destroy` retires it; repeated release returns NotFound.
IDs also remain stable through compaction and are never used as array indices.
Calls serialize and can be used from multiple threads, but snapshots cannot race
with the caller freeing its own buffers. C# bridge disposal serializes with calls.

Every output pointer must be valid, aligned and writable, every input pointer
valid/readable for the declared length. Buffers/outputs may not overlap. Null
pointers, alignment and lengths are checked; arbitrary dangling addresses cannot
be proven valid by the callee. No pointer is retained or borrowed across calls.

After failure, call `me_last_error(NULL,0,&required)` then allocate `required`
bytes and call again. Text is UTF-8 with a trailing NUL. These reads preserve
the error; the next ordinary successful call clears it. Errors are thread-local:
read on the same thread. Null error output returns Invalid. Rust panics become
Panic status; the build must use unwind. OOM/abort/foreign termination are not
recoverable through this protocol.

## Snapshot protocol

1. Query `me_snapshot_size` for bytes.
2. Allocate caller-owned storage.
3. `me_snapshot_copy` writes a complete snapshot and required bytes.
4. Capacity means retry with the returned size. No partial snapshot is published.
5. Read header and records from that successful copy only.

All wire numbers are little-endian; this is deliberately serialized rather than
copying potentially uninitialized Rust struct padding. C/C# layouts match it on
supported little-endian platforms. Big-endian native struct decoding is unsupported.

| Record | Offset | Type / meaning |
|---|---:|---|
| Header | 0 / 4 | u32 ABI / header bytes (1 / 40) |
| Header | 8 / 12 | u32 atom count / bond count |
| Header | 16 / 24 | u64 state / topology versions |
| Header | 32 / 36 | u32 atom stride / bond stride (40 / 16) |
| Atom | 0 / 12 | float32[3] position / velocity |
| Atom | 24 / 28 | u32 atomic number / reserved zero |
| Atom | 32 | u64 stable molecule ID |
| Bond | 0 / 4 | u32 indices into current snapshot's atom records |
| Bond | 8 / 12 | u32 order (0 unknown) / reserved zero |

Atoms follow the header; bonds follow atoms. There are no additional alignment
gaps. Molecule COM/velocity/mass and element CPK/radius/mass have separate checked
query APIs. Unity does not infer chemical identity or recompute COM dynamics.

No experimental zero-copy API is exposed in v1. A future zero-copy interface
needs an explicit borrow/release token and independently owned immutable storage.
ABI evolution must preserve v1 symbols/semantics or introduce a versioned ABI;
changing record strides silently is forbidden.

## v1.2 additive API (ABI remains 1)

`me_set_angular_velocity` sets world omega in rad/fs. `me_observables` returns
a 40-byte repr(C) record: four f64 values (KE, PE, total, temperature), u32 DOF
at offset 32 and u32 potential-availability flag at 36. Energies kcal/mol,
temperature K. When flag=0, PE/total numeric zeros are unused and must display
unavailable. Rust/C/C# layouts and actual DLL clients test the record.
Snapshot layout is unchanged; atom velocity now includes omega cross r, while
`me_molecule_info` still returns COM velocity. Thermal assigns translation and
rotation, so clients must not confuse a single atom velocity with COM velocity.

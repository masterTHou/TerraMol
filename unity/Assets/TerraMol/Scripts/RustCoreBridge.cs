using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

namespace TerraMol {
    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    public struct SnapshotHeader {
        public uint AbiVersion, HeaderBytes, AtomCount, BondCount;
        public ulong StateVersion, TopologyVersion;
        public uint AtomStride, BondStride;
    }
    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    public struct RenderAtom {
        public float X, Y, Z, Vx, Vy, Vz;
        public uint Element, Reserved;
        public ulong MoleculeId;
    }
    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    public struct RenderBond { public uint A, B, Order, Reserved; }
    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    public struct ElementInfo { public uint Number; public float R, G, B; public double Vdw, Mass; }
    [StructLayout(LayoutKind.Sequential, Pack = 8)]
    public struct MoleculeInfo { public double X, Y, Z, Vx, Vy, Vz, Mass; }
    public sealed class RenderSnapshot {
        public SnapshotHeader Header;
        public RenderAtom[] Atoms = new RenderAtom[0];
        public RenderBond[] Bonds = new RenderBond[0];
    }

    // Contains no Unity types: this exact source is tested in a standalone CLR process.
    public sealed class RustCoreBridge : IDisposable {
        const string Library = "mol_engine_core";
        readonly object gate = new object();
        ulong handle;
        byte[] bytes = new byte[0];
        readonly RenderSnapshot snapshot = new RenderSnapshot();
        readonly Dictionary<uint, ElementInfo> elements = new Dictionary<uint, ElementInfo>();
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_abi_version(out uint version);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_create(double size, double dt, out ulong handle);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_destroy(ulong h);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_step(ulong h, uint steps);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_reset_template(ulong h, uint kind, uint count, ulong seed, double kelvin);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_batch_spawn(ulong h, uint kind, uint count, ulong seed, uint retries);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_remove(ulong h, ulong id);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_clear(ulong h);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_set_velocity(ulong h, ulong id, double x, double y, double z);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_thermal(ulong h, double kelvin, ulong seed);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_flow(ulong h, double x, double y, double z);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_snapshot_size(ulong h, out ulong size);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_snapshot_copy(ulong h, [Out] byte[] buffer, ulong capacity, out ulong required);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_last_error([Out] byte[] buffer, ulong capacity, out ulong required);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_element(uint number, out ElementInfo info);
        [DllImport(Library, CallingConvention = CallingConvention.Cdecl)] static extern int me_molecule_info(ulong h, ulong id, out MoleculeInfo info);

        public static void ValidateLayout() {
            if (!BitConverter.IsLittleEndian || Marshal.SizeOf(typeof(SnapshotHeader)) != 40 ||
                Marshal.SizeOf(typeof(RenderAtom)) != 40 || Marshal.SizeOf(typeof(RenderBond)) != 16 ||
                Marshal.OffsetOf(typeof(RenderAtom), "MoleculeId").ToInt32() != 32 ||
                Marshal.OffsetOf(typeof(RenderAtom), "Element").ToInt32() != 24 ||
                Marshal.OffsetOf(typeof(SnapshotHeader), "StateVersion").ToInt32() != 16 ||
                Marshal.SizeOf(typeof(ElementInfo)) != 32 || Marshal.SizeOf(typeof(MoleculeInfo)) != 56)
                throw new PlatformNotSupportedException("TerraMol ABI v1 layout mismatch");
        }
        public RustCoreBridge(double size = 100, double dtFs = 1) {
            ValidateLayout(); uint version; Check(me_abi_version(out version));
            if (version != 1) throw new NotSupportedException("Unsupported TerraMol ABI");
            Check(me_create(size, dtFs, out handle));
        }
        static void Check(int code) {
            if (code == 0) return;
            ulong n; me_last_error(null, 0, out n);
            var message = new byte[checked((int)n)];
            me_last_error(message, n, out n);
            throw new InvalidOperationException("Rust status " + code + ": " + Encoding.UTF8.GetString(message).TrimEnd('\0'));
        }
        void Alive() { if (handle == 0) throw new ObjectDisposedException("RustCoreBridge"); }
        public void Step(uint steps) { lock (gate) { Alive(); Check(me_step(handle, steps)); } }
        public void Reset(uint kind, uint count, ulong seed, double kelvin) { lock (gate) { Alive(); Check(me_reset_template(handle, kind, count, seed, kelvin)); } }
        public void Spawn(uint kind, uint count, ulong seed) { lock (gate) { Alive(); Check(me_batch_spawn(handle, kind, count, seed, 1000)); } }
        public void Remove(ulong id) { lock (gate) { Alive(); Check(me_remove(handle, id)); } }
        public void Clear() { lock (gate) { Alive(); Check(me_clear(handle)); } }
        public void SetVelocity(ulong id, double x, double y, double z) { lock (gate) { Alive(); Check(me_set_velocity(handle, id, x, y, z)); } }
        public void Thermal(double kelvin, ulong seed) { lock (gate) { Alive(); Check(me_thermal(handle, kelvin, seed)); } }
        public void Flow(double x, double y, double z) { lock (gate) { Alive(); Check(me_flow(handle, x, y, z)); } }
        public ElementInfo Element(uint number) { lock (gate) { Alive(); ElementInfo info; if (!elements.TryGetValue(number, out info)) { Check(me_element(number, out info)); elements.Add(number, info); } return info; } }
        public MoleculeInfo Molecule(ulong id) { lock (gate) { Alive(); MoleculeInfo info; Check(me_molecule_info(handle, id, out info)); return info; } }
        // Returned object is borrowed until the next ReadSnapshot; UI consumes it synchronously.
        public RenderSnapshot ReadSnapshot() {
            lock (gate) {
                Alive(); ulong n; Check(me_snapshot_size(handle, out n));
                for (int attempt = 0; attempt < 3; ++attempt) {
                    if ((ulong)bytes.Length < n) bytes = new byte[checked((int)n)];
                    int status = me_snapshot_copy(handle, bytes, (ulong)bytes.Length, out n);
                    if (status == 3) continue;
                    Check(status);
                    var pin = GCHandle.Alloc(bytes, GCHandleType.Pinned);
                    try {
                        var p = pin.AddrOfPinnedObject();
                        var h = (SnapshotHeader)Marshal.PtrToStructure(p, typeof(SnapshotHeader));
                        if (h.AbiVersion != 1 || h.HeaderBytes != 40 || h.AtomStride != 40 || h.BondStride != 16 ||
                            n != 40UL + h.AtomCount * 40UL + h.BondCount * 16UL) throw new InvalidOperationException("Invalid snapshot layout");
                        if (snapshot.Atoms.Length != h.AtomCount) snapshot.Atoms = new RenderAtom[checked((int)h.AtomCount)];
                        if (snapshot.Bonds.Length != h.BondCount) snapshot.Bonds = new RenderBond[checked((int)h.BondCount)];
                        int offset = 40;
                        for (int i = 0; i < snapshot.Atoms.Length; ++i, offset += 40) snapshot.Atoms[i] = (RenderAtom)Marshal.PtrToStructure(IntPtr.Add(p, offset), typeof(RenderAtom));
                        for (int i = 0; i < snapshot.Bonds.Length; ++i, offset += 16) {
                            var b = (RenderBond)Marshal.PtrToStructure(IntPtr.Add(p, offset), typeof(RenderBond));
                            if (b.A >= h.AtomCount || b.B >= h.AtomCount) throw new InvalidOperationException("Invalid bond index");
                            snapshot.Bonds[i] = b;
                        }
                        snapshot.Header = h; return snapshot;
                    } finally { pin.Free(); }
                }
                throw new InvalidOperationException("Snapshot topology changed repeatedly; retry next frame");
            }
        }
        public void Dispose() { lock (gate) { if (handle != 0) { me_destroy(handle); handle = 0; } } GC.SuppressFinalize(this); }
        ~RustCoreBridge() { try { Dispose(); } catch { /* No exception may escape finalization. */ } }
    }
}

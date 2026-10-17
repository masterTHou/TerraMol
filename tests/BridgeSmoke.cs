using System;
using TerraMol;
public static class BridgeSmoke {
    static void Assert(bool condition) { if(!condition)throw new Exception("Bridge assertion failed"); }
    public static void Main() {
        RustCoreBridge.ValidateLayout();
        using(var core=new RustCoreBridge()) {
            core.Spawn(0,50,42);var s=core.ReadSnapshot();Assert(s.Atoms.Length==150 && s.Bonds.Length==100);
            ulong id=s.Atoms[0].MoleculeId;var m=core.Molecule(id);Assert(m.Mass>18 && m.Mass<19);
            core.SetVelocity(id,0.01,0,0);core.Step(10);s=core.ReadSnapshot();Assert(s.Atoms[0].Vx==0.01f);
            core.SetAngularVelocity(id,0,0.02,0);core.Step(10);
            var observables=core.Observables();Assert(observables.Kinetic>0 && observables.DegreesOfFreedom==300 && observables.HasPotential==0);
            core.Remove(id);s=core.ReadSnapshot();Assert(s.Atoms.Length==147);
            foreach(var b in s.Bonds)Assert(b.A<147 && b.B<147);
            core.Clear();Assert(core.ReadSnapshot().Atoms.Length==0);
            core.Reset(0,50,42,300);s=core.ReadSnapshot();var first=s.Atoms[0];
            core.Step(100);core.Reset(0,50,42,300);s=core.ReadSnapshot();
            Assert(first.X==s.Atoms[0].X && first.Vx==s.Atoms[0].Vx && first.MoleculeId!=s.Atoms[0].MoleculeId);
        }
        var disposed=new RustCoreBridge();disposed.Dispose();disposed.Dispose();
        try{disposed.Step(1);throw new Exception("Missing disposed check");}catch(ObjectDisposedException){}
        Console.WriteLine("C# / Rust actual DLL layout, snapshots and lifecycle OK");
    }
}

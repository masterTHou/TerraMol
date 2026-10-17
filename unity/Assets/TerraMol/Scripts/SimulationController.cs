using System;
using System.Globalization;
using UnityEngine;
namespace TerraMol {
    [RequireComponent(typeof(MoleculeVisualizer),typeof(MoleculePicker))]
    public sealed class SimulationController : MonoBehaviour {
        public double BoxSize=100, TimestepFs=1;
        public float StepsPerSecond=1000, Speed=1;
        public bool Running;
        public RustCoreBridge Core { get; private set; }
        public MoleculeVisualizer Visualizer { get; private set; }
        MoleculePicker picker;
        double accumulator;
        int kind;
        ulong seed=42;
        string vx="0.01",vy="0",vz="0",temperature="300",message="Native: rigid spheres; no atomistic force field";
        readonly Rect panel=new Rect(12,12,300,430);
        void OnEnable() {
            Visualizer=GetComponent<MoleculeVisualizer>();picker=GetComponent<MoleculePicker>();picker.Controller=this;
            Visualizer.ResetVersions();
            try { Core=new RustCoreBridge(BoxSize,TimestepFs); Visualizer.Render(Core.ReadSnapshot(),Core); }
            catch(Exception e) {message=e.Message;Debug.LogException(e);}
        }
        void Update() {
            if(Core==null)return;
            try {
                if(Running) {
                    accumulator += Time.unscaledDeltaTime * StepsPerSecond * Speed;
                    uint steps=(uint)Math.Min(200,Math.Floor(accumulator));
                    if(steps>0){Core.Step(steps);accumulator-=steps;}
                    // Cap backlog after a long editor stall; physical dt itself never changes.
                    accumulator=Math.Min(accumulator,200);
                }
                Visualizer.Render(Core.ReadSnapshot(),Core);
            } catch(Exception e){Running=false;message=e.Message;}
        }
        public bool DebugPanelContains(Vector3 mouse) { return panel.Contains(new Vector2(mouse.x,Screen.height-mouse.y)); }
        void InvokeAction(Action action){try{action();message="Done";}catch(Exception e){message=e.Message;}}
        static double Number(string s){return double.Parse(s,CultureInfo.InvariantCulture);}
        void OnGUI() {
            GUILayout.BeginArea(panel,GUI.skin.box);
            GUILayout.Label("TerraMol | Native rigid-body prototype");
            GUILayout.Label("Angstrom / fs / amu / K");
            kind=GUILayout.SelectionGrid(kind,new[]{"Water","Methane","Benzene"},3);
            GUI.enabled=Core!=null;
            if(GUILayout.Button("Spawn one"))InvokeAction(()=>Core.Spawn((uint)kind,1,seed++));
            if(GUILayout.Button("Spawn 50"))InvokeAction(()=>Core.Spawn((uint)kind,50,seed++));
            if(GUILayout.Button("Delete selected"))InvokeAction(()=>{Core.Remove(picker.SelectedId);picker.Select(0);});
            if(GUILayout.Button("Clear"))InvokeAction(()=>{Core.Clear();picker.Select(0);});
            GUILayout.Label("Selected molecule: "+picker.SelectedId);
            GUILayout.BeginHorizontal();vx=GUILayout.TextField(vx);vy=GUILayout.TextField(vy);vz=GUILayout.TextField(vz);GUILayout.EndHorizontal();
            if(GUILayout.Button("Set COM velocity (A/fs)"))InvokeAction(()=>Core.SetVelocity(picker.SelectedId,Number(vx),Number(vy),Number(vz)));
            GUILayout.BeginHorizontal();temperature=GUILayout.TextField(temperature);
            if(GUILayout.Button("Thermal (K)"))InvokeAction(()=>Core.Thermal(Number(temperature),seed++));GUILayout.EndHorizontal();
            Running=GUILayout.Toggle(Running,"Run fixed physical timesteps");
            GUILayout.Label("Playback speed: "+Speed.ToString("F1")+"x");Speed=GUILayout.HorizontalSlider(Speed,0.1f,10f);
            GUILayout.Label("dt = "+TimestepFs+" fs; capped at 200 steps/frame");
            GUI.enabled=true;GUILayout.Label(message);GUILayout.EndArea();
        }
        void Release(){Running=false;accumulator=0;if(Core!=null){Core.Dispose();Core=null;}}
        void OnDisable(){Release();}void OnDestroy(){Release();}void OnApplicationQuit(){Release();}
    }
}

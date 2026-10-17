using UnityEngine;
namespace TerraMol {
    public sealed class MoleculePicker : MonoBehaviour {
        public SimulationController Controller;
        public Camera ViewCamera;
        public ulong SelectedId { get; private set; }
        LineRenderer arrow;
        public void Select(ulong id) { SelectedId=id; Controller.Visualizer.Select(id); }
        void Start() {
            if (ViewCamera == null) ViewCamera=Camera.main;
            var go=new GameObject("Selected COM velocity (Angstrom/fs x 200)");go.transform.SetParent(Controller.Visualizer.transform,false);
            arrow=go.AddComponent<LineRenderer>();arrow.useWorldSpace=false;arrow.positionCount=5;
            arrow.startWidth=0.12f;arrow.endWidth=0.12f;arrow.sharedMaterial=Controller.Visualizer.SharedAtomMaterial;
            arrow.startColor=Color.yellow;arrow.endColor=Color.yellow;arrow.enabled=false;
        }
        void Update() {
            if (Controller.Core == null || ViewCamera == null) return;
            if (Input.GetMouseButtonDown(0) && !Controller.DebugPanelContains(Input.mousePosition)) {
                RaycastHit hit;
                if (Physics.Raycast(ViewCamera.ScreenPointToRay(Input.mousePosition),out hit)) {
                    var atom=hit.collider.GetComponent<AtomView>(); if(atom != null) Select(atom.MoleculeId);
                }
            }
        }
        void LateUpdate() {
            if (arrow == null || Controller.Core == null) return;
            if (SelectedId == 0 || !Controller.Visualizer.Contains(SelectedId)) { Select(0);arrow.enabled=false;return; }
            var m=Controller.Core.Molecule(SelectedId);var start=new Vector3((float)m.X,(float)m.Y,(float)m.Z);
            var delta=new Vector3((float)m.Vx,(float)m.Vy,(float)m.Vz)*200f;
            arrow.enabled=delta.sqrMagnitude>1e-10f;if(!arrow.enabled)return;
            var end=start+delta;var side=Vector3.Cross(delta.normalized,Vector3.up);
            if(side.sqrMagnitude<0.01f)side=Vector3.right;side=side.normalized*Mathf.Min(1f,delta.magnitude*0.2f);
            var back=delta.normalized*Mathf.Min(2f,delta.magnitude*0.3f);
            arrow.SetPosition(0,start);arrow.SetPosition(1,end);arrow.SetPosition(2,end-back+side);arrow.SetPosition(3,end);arrow.SetPosition(4,end-back-side);
        }
    }
}

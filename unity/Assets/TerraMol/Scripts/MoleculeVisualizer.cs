using System.Collections.Generic;
using UnityEngine;
namespace TerraMol {
    public sealed class MoleculeVisualizer : MonoBehaviour {
        readonly List<AtomView> atoms = new List<AtomView>();
        readonly List<Transform> bonds = new List<Transform>();
        Material atomMaterial, bondMaterial;
        ulong topology = ulong.MaxValue, state = ulong.MaxValue, selected;
        public void ResetVersions() { topology=ulong.MaxValue;state=ulong.MaxValue;selected=0; }
        public Material SharedAtomMaterial { get { EnsureMaterials(); return atomMaterial; } }
        static Material NewMaterial(Color color) {
            var shader = Shader.Find("Universal Render Pipeline/Lit");
            if (shader == null) shader = Shader.Find("Standard");
            var m = new Material(shader); m.SetColor("_Color", color); m.SetColor("_BaseColor", color); return m;
        }
        void EnsureMaterials() { if (atomMaterial == null) atomMaterial = NewMaterial(Color.white); if (bondMaterial == null) bondMaterial = NewMaterial(new Color(0.45f,0.5f,0.6f)); }
        public void Select(ulong id) { selected = id; foreach (var a in atoms) if (a.gameObject.activeSelf) a.Highlight(a.MoleculeId == selected); }
        public bool Contains(ulong id) { foreach (var a in atoms) if (a.gameObject.activeSelf && a.MoleculeId == id) return true; return false; }
        public void Render(RenderSnapshot snapshot, RustCoreBridge core) {
            EnsureMaterials(); var h = snapshot.Header;
            if (h.TopologyVersion != topology) {
                while (atoms.Count < snapshot.Atoms.Length) {
                    var go = GameObject.CreatePrimitive(PrimitiveType.Sphere); go.name = "Atom";
                    go.transform.SetParent(transform, false); atoms.Add(go.AddComponent<AtomView>());
                }
                while (bonds.Count < snapshot.Bonds.Length) {
                    var go = GameObject.CreatePrimitive(PrimitiveType.Cylinder); go.name = "Bond";
                    Destroy(go.GetComponent<Collider>()); go.transform.SetParent(transform, false);
                    go.GetComponent<Renderer>().sharedMaterial = bondMaterial; bonds.Add(go.transform);
                }
                for (int i = 0; i < atoms.Count; ++i) {
                    atoms[i].gameObject.SetActive(i < snapshot.Atoms.Length);
                    if (i < snapshot.Atoms.Length) atoms[i].Bind(snapshot.Atoms[i], core.Element(snapshot.Atoms[i].Element), atomMaterial);
                }
                for (int i = 0; i < bonds.Count; ++i) bonds[i].gameObject.SetActive(i < snapshot.Bonds.Length);
                topology = h.TopologyVersion; Select(selected);
            } else if (h.StateVersion == state) return;
            for (int i = 0; i < snapshot.Atoms.Length; ++i) atoms[i].transform.localPosition = Position(snapshot.Atoms[i]);
            for (int i = 0; i < snapshot.Bonds.Length; ++i) {
                var b = snapshot.Bonds[i]; var a = Position(snapshot.Atoms[b.A]); var z = Position(snapshot.Atoms[b.B]);
                Vector3 d = z-a; var t = bonds[i]; t.localPosition = (a+z)*0.5f;
                t.localRotation = d.sqrMagnitude > 1e-12f ? Quaternion.FromToRotation(Vector3.up, d) : Quaternion.identity;
                t.localScale = new Vector3(0.16f, d.magnitude*0.5f, 0.16f);
            }
            state = h.StateVersion;
        }
        static Vector3 Position(RenderAtom a) { return new Vector3(a.X,a.Y,a.Z); }
        void OnDestroy() { if (atomMaterial != null) Destroy(atomMaterial); if (bondMaterial != null) Destroy(bondMaterial); }
    }
}

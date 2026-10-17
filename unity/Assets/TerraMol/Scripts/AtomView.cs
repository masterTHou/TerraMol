using UnityEngine;
namespace TerraMol {
    public sealed class AtomView : MonoBehaviour {
        public ulong MoleculeId { get; private set; }
        Renderer meshRenderer;
        MaterialPropertyBlock properties;
        Color baseColor;
        void Awake() { meshRenderer = GetComponent<Renderer>(); properties = new MaterialPropertyBlock(); }
        public void Bind(RenderAtom atom, ElementInfo element, Material shared) {
            MoleculeId = atom.MoleculeId;
            meshRenderer.sharedMaterial = shared;
            baseColor = new Color(element.R, element.G, element.B);
            transform.localScale = Vector3.one * (float)(element.Vdw * 0.6);
            Highlight(false);
        }
        public void Highlight(bool selected) {
            Color color = selected ? Color.Lerp(baseColor, Color.yellow, 0.65f) : baseColor;
            properties.SetColor("_Color", color); properties.SetColor("_BaseColor", color);
            meshRenderer.SetPropertyBlock(properties);
        }
    }
}

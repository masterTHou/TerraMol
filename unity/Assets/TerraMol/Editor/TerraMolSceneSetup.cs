using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
namespace TerraMol {
    public static class TerraMolSceneSetup {
        [MenuItem("TerraMol/Create prototype scene")]
        public static void Create() {
            var scene=EditorSceneManager.NewScene(NewSceneSetup.EmptyScene,NewSceneMode.Single);
            var root=new GameObject("TerraMol Rust Simulation");root.AddComponent<SimulationController>();
            var camera=new GameObject("Main Camera");camera.tag="MainCamera";
            var cam=camera.AddComponent<Camera>();cam.backgroundColor=new Color(0.025f,0.04f,0.08f);cam.clearFlags=CameraClearFlags.SolidColor;
            camera.transform.position=new Vector3(50,60,-90);camera.transform.LookAt(new Vector3(50,50,50));
            var light=new GameObject("Directional Light").AddComponent<Light>();light.type=LightType.Directional;light.transform.rotation=Quaternion.Euler(35,-30,0);
            RenderSettings.ambientLight=new Color(0.4f,0.4f,0.45f);
            System.IO.Directory.CreateDirectory("Assets/TerraMol/Scenes");
            EditorSceneManager.SaveScene(scene,"Assets/TerraMol/Scenes/Prototype.unity");
            Selection.activeGameObject=root;
        }
    }
}

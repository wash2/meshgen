using UnityEngine;
using System.Collections;
using UnityEditor;

[CustomEditor (typeof (ChunkGenerator))]
public class ChunkEditor : Editor {

	public override void OnInspectorGUI() {
		ChunkGenerator chunkgen = (ChunkGenerator)target;

		if (DrawDefaultInspector ()) {
			if (chunkgen.autoUpdate) {
				chunkgen.GenerateChunk();
			}
		}

		if (GUILayout.Button ("Generate")) {
			chunkgen.GenerateChunk();
		}
	}
}
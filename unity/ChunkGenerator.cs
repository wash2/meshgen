using UnityEngine;
using UnityEngine.Rendering;
using Unity.Collections.LowLevel.Unsafe;
using Unity.Collections;

using System;
using System.Runtime.InteropServices;
using System.Collections;
using System.Collections.Generic;

public class ChunkGenerator : MonoBehaviour
{
    [DllImport("meshgen")]
    private static extern void init_logger();

    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunkgen(UIntPtr side_len, double height);
    [DllImport("meshgen")]
    private static extern void free_mountainous_terrain_chunkgen(IntPtr chunkgen);
    [DllImport("meshgen")]
    private static extern IntPtr fill_mountainous_terrain_chunk(IntPtr chunkgen, IntPtr vbuf, IntPtr ibuf, IntPtr tbuf, IntPtr pos);
    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunk_geometry_desc(IntPtr chunkgen, out int vCnt, out int eCnt, out int fCnt);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_chunkgen_dim(IntPtr chunkgen, UIntPtr sideLength, double height);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_chunkgen_noise(IntPtr chunkgen, uint seed, uint octaves, double scale, double persistence, double lacunarity, double displacement, double bias_gain_a, IntPtr bezier_from, IntPtr bezier_to, double bezier_bias_control);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_chunkgen_color_gradient(IntPtr chunkgen, IntPtr colorKeys, UIntPtr keyCnt, bool isLinearBlend);

    [StructLayout(LayoutKind.Sequential)]
    struct ExampleVertex
    {
        public Vector3 pos;
        // public Vector3 normal;
        // public Vector4 tangent;
        public Vector2 uv;
    }

    IntPtr chunkgen;
    NativeArray<ExampleVertex> verts;
    NativeArray<int> tris;
    int vertexCount;
    int edgeCount;
    int faceCount;
    Mesh mesh;

    IntPtr texturegen; 

	public bool autoUpdate;
    uint sideLength = 128;
    public uint seed = 0;
    [Range(0,8)]
    public uint octaves = 3;
    public double scale = 10;
    [Range(0,1)]
    public double persistence = 0.5;
    [Range(0,8)]
    public double lacunarity = 0.5;
    [Range(-.1f, 2)]
    public double displacement = -.1;
    [Range(0,1)]
    public double bias_gain_a = .5;
    [Range(0,1)]
    public double bezier_bias_control = .5;
    public Vector2 bezier_bias_from = new Vector2(.4f, .01f);
    public Vector2 bezier_bias_to = new Vector2(.55f, .15f);
    public double height = 50;
    public Vector3 offset;
    public Material m_Material;
    public float animationSpeed = 1;
    Texture2D texture;
    MeshCollider meshCollider;

    NativeArray<Color32> tex_data;
    public CustomGradient colorGradients;
    public AnimationCurve heightMapControl;

    void Start()
    {
        init_logger();

        offset = new Vector3();
        mesh = new Mesh();
        meshCollider = gameObject.AddComponent<MeshCollider>();
        texture = new Texture2D ((int)sideLength + 1, (int)sideLength + 1);
        tex_data = texture.GetRawTextureData<Color32>();
        gameObject.AddComponent<MeshFilter>();
        GetComponent<MeshFilter>().sharedMesh = mesh;

        gameObject.AddComponent<MeshRenderer>();
        GetComponent<MeshRenderer>().sharedMaterial = m_Material;

        chunkgen = get_mountainous_terrain_chunkgen((UIntPtr)sideLength, height);
        var res = Marshal.PtrToStringAnsi(
            get_mountainous_terrain_chunk_geometry_desc(chunkgen, out vertexCount, out edgeCount, out faceCount)
        );
        if (!res.Equals("OK")) {
            return;
        }
        verts = new NativeArray<ExampleVertex>(vertexCount, Allocator.Persistent);
        tris = new NativeArray<int>(faceCount * 3, Allocator.Persistent);
        Debug.Log(chunkgen);

        GenerateChunk();
    }

    void Update() {
        offset += new Vector3(animationSpeed * Time.deltaTime, 0, 0);
        GenerateChunk();
    }

    public void GenerateChunk()
    {
        var t0 = Time.realtimeSinceStartup;
        set_mountainous_terrain_chunkgen_dim(chunkgen, (UIntPtr)sideLength, height);
        unsafe {
            fixed(Vector2* bezier_bias_from_ptr = &bezier_bias_from, bezier_bias_to_ptr = &bezier_bias_to) {
                set_mountainous_terrain_chunkgen_noise(chunkgen, seed, octaves, scale, persistence, lacunarity, displacement, bias_gain_a, new IntPtr(bezier_bias_from_ptr), new IntPtr(bezier_bias_to_ptr), bezier_bias_control);
            }
        }

        var layout = new[]
        {
            new VertexAttributeDescriptor(VertexAttribute.Position, VertexAttributeFormat.Float32, 3),
            // new VertexAttributeDescriptor(VertexAttribute.Normal, VertexAttributeFormat.Float32, 3),
            // new VertexAttributeDescriptor(VertexAttribute.Tangent, VertexAttributeFormat.Float32, 4),
            new VertexAttributeDescriptor(VertexAttribute.TexCoord0, VertexAttributeFormat.Float32, 2),
        };

        mesh.Clear();
        var colorKeys = new NativeArray<CustomGradient.ColourKey>(colorGradients.NumKeys, Allocator.Temp);
        var colorKeysArr = colorGradients.ToArray();
        colorKeys.CopyFrom(colorKeysArr);

        unsafe {
            set_mountainous_terrain_chunkgen_color_gradient(
                chunkgen, 
                new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(colorKeys)), 
                (UIntPtr)colorGradients.NumKeys, 
                colorGradients.blendMode == CustomGradient.BlendMode.Linear
            );
            fixed(Vector3* offset_ptr = &offset) {
                var res = Marshal.PtrToStringAnsi(
                    fill_mountainous_terrain_chunk(chunkgen, new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(verts)), new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(tris)), new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(tex_data)), new IntPtr(offset_ptr))
                );
                if (!res.Equals("OK")) {
                    Debug.Log(res);
                    return;
                }
            }
            mesh.SetVertexBufferParams(vertexCount, layout);
            mesh.SetIndexBufferParams(faceCount * 3, UnityEngine.Rendering.IndexFormat.UInt32);

            mesh.SetVertexBufferData(verts, 0, 0, vertexCount, 0, MeshUpdateFlags.DontRecalculateBounds | MeshUpdateFlags.DontValidateIndices);
            mesh.SetIndexBufferData(tris, 0, 0, faceCount * 3);

            mesh.subMeshCount = 1;

            var meshDesc = new UnityEngine.Rendering.SubMeshDescriptor(0, faceCount * 3, MeshTopology.Triangles);
            mesh.SetSubMesh(0, meshDesc);
            mesh.bounds = new Bounds(Vector3.zero, new Vector3(sideLength * 2, (float)height * 2, sideLength * 2));
            mesh.RecalculateNormals();
            texture.Apply();

            GetComponent<MeshRenderer>().material.SetTexture("_BaseColorMap", texture);
 
            // Finaly we set the Mesh in the MeshCollider
            meshCollider.sharedMesh = mesh;
            Debug.Log(Time.realtimeSinceStartup - t0);
        }
    }

    void OnApplicationQuit()
    {
        free_mountainous_terrain_chunkgen(chunkgen);
        verts.Dispose();
        tris.Dispose();
        Debug.Log("freed chunkgen");
    }

    void OnValidate() {
        if (sideLength < 1) {
            sideLength = 1;
        }
        if (lacunarity < 1) {
            lacunarity = 1;
        }
        if (octaves < 0) {
            octaves = 0;
        }
    }
}
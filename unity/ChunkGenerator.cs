using System.Collections;
using System.Collections.Generic;
using UnityEngine.Rendering;
using UnityEngine;
using Unity.Collections.LowLevel.Unsafe;
using Unity.Collections;
using System.Runtime.InteropServices;
using System;

public class ChunkGenerator : MonoBehaviour
{
    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunkgen(UIntPtr side_len, double height, uint seed, uint octaves, double scale, double persistence, double lacunarity, double displacement, double bias_gain_a);
    [DllImport("meshgen")]
    private static extern void free_mountainous_terrain_chunkgen(IntPtr chunkgen);
    [DllImport("meshgen")]
    private static extern IntPtr fill_mountainous_terrain_chunk(IntPtr chunkgen, IntPtr vbuf, IntPtr ibuf, IntPtr pos);
    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunk_geometry_desc(IntPtr chunkgen, out int vCnt, out int eCnt, out int fCnt);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_chunkgen_dim(IntPtr chunkgen, UIntPtr sideLength, double height);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_chunkgen_noise(IntPtr chunkgen, uint seed, uint octaves, double scale, double persistence, double lacunarity, double displacement, double bias_gain_a);

    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_texturegen(UIntPtr width, UIntPtr height,  uint seed, uint octaves, double scale, double persistence, double lacunarity, double displacement, double bias_gain_a);
    [DllImport("meshgen")]
    private static extern void free_mountainous_terrain_texturegen(IntPtr texturegen);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_texturegen_dim(IntPtr texturegen, UIntPtr width, UIntPtr height);
    [DllImport("meshgen")]
    private static extern void set_mountainous_terrain_texturegen_noise(IntPtr texturegen, uint seed, uint octaves, double scale, double persistence, double lacunarity, double displacement, double bias_gain_a);
    [DllImport("meshgen")]
    private static extern IntPtr fill_mountainous_terrain_texture_2d(IntPtr texturegen, IntPtr tbuf, IntPtr pos);

    [StructLayout(LayoutKind.Sequential)]
    struct ExampleVertex
    {
        public Vector3 pos;
        public Vector3 normal;
        public Vector4 tangent;
        public Vector2 uv;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct ColorKeyMessage
    {
        public IntPtr colorKeys;
        public UIntPtr size;
        public bool blendMode;
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
    uint sideLength = 127;
    public uint seed = 0;
    [Range(0,8)]
    public uint octaves = 3;
    public double scale = 10;
    [Range(0,1)]
    public double persistence = 0.5;
    [Range(0,8)]
    public double lacunarity = 0.5;
    public double displacement = -1.0;
    [Range(0,1)]
    public double bias_gain_a = .5;
    public double height = 50;
    Vector3 offset;
    public Material m_Material;
    public float animationSpeed = 1;
    Texture2D texture;

    void Start()
    {
        offset = new Vector3();
        mesh = new Mesh();
        chunkgen = get_mountainous_terrain_chunkgen((UIntPtr)sideLength, height, seed, octaves, scale, persistence, lacunarity, displacement, bias_gain_a);
        var res = Marshal.PtrToStringAnsi(
            get_mountainous_terrain_chunk_geometry_desc(chunkgen, out vertexCount, out edgeCount, out faceCount)
        );
        if (!res.Equals("OK")) {
            return;
        }
        verts = new NativeArray<ExampleVertex>(vertexCount, Allocator.Persistent);
        tris = new NativeArray<int>(faceCount * 3, Allocator.Persistent);
        Debug.Log(chunkgen);

        texturegen = get_mountainous_terrain_texturegen((UIntPtr)sideLength, (UIntPtr)sideLength, seed, octaves, scale, persistence, lacunarity, displacement, bias_gain_a);

        gameObject.AddComponent<MeshFilter>();
        gameObject.AddComponent<MeshRenderer>();
        GetComponent<MeshFilter>().mesh = mesh;
        GetComponent<MeshRenderer>().material = m_Material;
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
        set_mountainous_terrain_chunkgen_noise(chunkgen, seed, octaves, scale, persistence, lacunarity, displacement, bias_gain_a);
        
        var layout = new[]
        {
            new VertexAttributeDescriptor(VertexAttribute.Position, VertexAttributeFormat.Float32, 3),
            new VertexAttributeDescriptor(VertexAttribute.Normal, VertexAttributeFormat.Float32, 3),
            new VertexAttributeDescriptor(VertexAttribute.Tangent, VertexAttributeFormat.Float32, 4),
            new VertexAttributeDescriptor(VertexAttribute.TexCoord0, VertexAttributeFormat.Float32, 2),
        };

        mesh.Clear();
        
        // fill Unity allocated NativeArray
        unsafe {
            fixed(Vector3* offset_ptr = &offset) {
                var res = Marshal.PtrToStringAnsi(
                    fill_mountainous_terrain_chunk(chunkgen, new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(verts)), new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(tris)), new IntPtr(offset_ptr))
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

            Debug.Log(Time.realtimeSinceStartup - t0);
        }
    }

    Texture2D GetTexture(int width, int height) {
        texture = new Texture2D ((int)sideLength, (int)sideLength);

        // update dim
        set_mountainous_terrain_texturegen_dim(texturegen, (UIntPtr)sideLength, (UIntPtr)sideLength);
        set_mountainous_terrain_texturegen_noise(texturegen, seed, octaves, scale, persistence, lacunarity, displacement, bias_gain_a);
        var data = texture.GetRawTextureData<Color32>();
        var offset2D = new Vector2(offset.x, offset.z);
        unsafe {
            Vector2* offser_ptr = &offset2D;
            fill_mountainous_terrain_texture_2d(texturegen, new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(data)), new IntPtr(offser_ptr));
        }
        return texture;
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
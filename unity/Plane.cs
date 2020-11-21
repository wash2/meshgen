using System.Collections;
using System.Collections.Generic;
using UnityEngine.Rendering;
using UnityEngine;
using Unity.Collections.LowLevel.Unsafe;
using Unity.Collections;
using System.Runtime.InteropServices;
using System;

public class Plane : MonoBehaviour
{
    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunkgen(UIntPtr side_len, uint seed, uint octaves, double scale, double persistence, double lacunarity);
    [DllImport("meshgen")]
    private static extern void free_mountainous_terrain_chunkgen(IntPtr chunkgen);
    [DllImport("meshgen")]
    private static extern IntPtr fill_mountainous_terrain_chunk(IntPtr chunkgen, IntPtr vbuf, IntPtr ibuf, IntPtr pos);
    [DllImport("meshgen")]
    private static extern IntPtr get_mountainous_terrain_chunk_geometry_desc(IntPtr chunkgen, out int vCnt, out int eCnt, out int fCnt);
    
    [System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
    struct ExampleVertex
    {
        public Vector3 pos;
        public Vector3 normal;
        public Vector4 tangent;
        public Vector2 uv;
    }

    Vector3[] newVertices;
    Vector2[] newUV;
    int[] newTriangles;
    IntPtr chunkgen;

    public uint sideLength = 100;
    public uint seed = 0;
    public uint octaves = 3;
    public double scale = 10;
    public double persistence = 1.5;
    public double lacunarity = 1.5;
    
    public Material m_Material;

    void Start()
    {
        var t0 = Time.realtimeSinceStartup;
        chunkgen = get_mountainous_terrain_chunkgen((UIntPtr)sideLength, seed, octaves, scale, persistence, lacunarity);

        var layout = new[]
        {
            new VertexAttributeDescriptor(VertexAttribute.Position, VertexAttributeFormat.Float32, 3),
            new VertexAttributeDescriptor(VertexAttribute.Normal, VertexAttributeFormat.Float32, 3),
            new VertexAttributeDescriptor(VertexAttribute.Tangent, VertexAttributeFormat.Float32, 4),
            new VertexAttributeDescriptor(VertexAttribute.TexCoord0, VertexAttributeFormat.Float32, 2),
        };

        gameObject.AddComponent<MeshFilter>();
        gameObject.AddComponent<MeshRenderer>();
        Mesh mesh = new Mesh();
        GetComponent<MeshFilter>().mesh = mesh;
        GetComponent<MeshRenderer>().material = m_Material;
        mesh.Clear();
        
        // fill Unity allocated NativeArray
        unsafe {
            int vertexCount;
            int edgeCount;
            int faceCount;
            var res = Marshal.PtrToStringAnsi(
                get_mountainous_terrain_chunk_geometry_desc(chunkgen, out vertexCount, out edgeCount, out faceCount)
            );
            if (!res.Equals("OK")) {
                return;
            }
            Debug.Log(vertexCount);

            var verts = new NativeArray<ExampleVertex>(vertexCount, Allocator.Temp);
            var tris = new NativeArray<int>(faceCount * 3, Allocator.Temp);
            Vector3 pos = new Vector3();
            Vector3* pos_ptr = &pos;
            res = Marshal.PtrToStringAnsi(
                fill_mountainous_terrain_chunk(chunkgen, new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(verts)), new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(tris)), new IntPtr(pos_ptr))
            );
            if (!res.Equals("OK")) {
                Debug.Log(res);
                return;
            }

            mesh.SetVertexBufferParams(vertexCount, layout);
            mesh.SetIndexBufferParams(faceCount * 3, UnityEngine.Rendering.IndexFormat.UInt32);

            mesh.SetVertexBufferData(verts, 0, 0, vertexCount);
            mesh.SetIndexBufferData(tris, 0, 0, faceCount * 3);

            mesh.subMeshCount = 1;

            var meshDesc = new UnityEngine.Rendering.SubMeshDescriptor(0, faceCount * 3, MeshTopology.Triangles);
            mesh.SetSubMesh(0, meshDesc);
            Debug.Log(Time.realtimeSinceStartup - t0);
        }
    }

    void OnApplicationQuit()
    {
        free_mountainous_terrain_chunkgen(chunkgen);
        Debug.Log("freed chunkgen");
    }
}
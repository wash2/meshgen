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
    private static extern IntPtr fill_plane(IntPtr vbuf, IntPtr ibuf, UIntPtr sideLength);
    [DllImport("meshgen")]
    private static extern IntPtr get_plane_desc(UIntPtr sideLength, out int vCnt, out int eCnt, out int fCnt);
    
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

    public UIntPtr sideLength = (UIntPtr)10000;
    public float frequency;
    public Material m_Material;

    void Start()
    {
        var t0 = Time.realtimeSinceStartup;

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
                get_plane_desc(sideLength, out vertexCount, out edgeCount, out faceCount)
            );
            if (!res.Equals("OK")) {
                Debug.Log(res);
                return;
            }

            var verts = new NativeArray<ExampleVertex>(vertexCount, Allocator.Temp);
            var tris = new NativeArray<int>(faceCount * 3, Allocator.Temp);
            res = Marshal.PtrToStringAnsi(
                fill_plane(new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(verts)), new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(tris)), sideLength)
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
        }

        // Marshal rust created buffers - Seems unlikely to work well....
        // figure out how to marshal without looping & PtrToStructure 
        // https://social.msdn.microsoft.com/Forums/vstudio/en-US/7bd09baf-f78e-462f-9d65-7c3324350493/marshal-struct-containing-pointertoarrayofstructs-from-c-to-c?forum=clr

                // int vertSide = sideLength + 1;
        // newVertices = new Vector3[vertSide * vertSide];
        // newUV = new Vector2[vertSide * vertSide];
        // for (int x = 0; x < vertSide; x++) {
        //     for (int z = 0; z < vertSide; z++) {
        //         int cur = x * vertSide + z;
        //         float v_x = -sideLength / 2f + x;
        //         float v_z =  -sideLength / 2f + z;
        //         newVertices[cur] = new Vector3(v_x, sideLength / 10 * Mathf.Cos(2 * Mathf.PI * frequency * (( Mathf.Abs(x- sideLength / 2f) + Mathf.Abs(z - sideLength / 2f)) / sideLength)), v_z);
        //         // Debug.Log(sideLength / 2 * Mathf.Cos(2 * Mathf.PI * (x + z - sideLength)));
        //     }
        // }

        // for (int x = 0; x < vertSide; x++) {
        //     for (int z = 0; z < vertSide; z++) {
        //         int cur = x * vertSide + z;
        //         newUV[cur] = new Vector2((float)x / vertSide, (float)z / vertSide);
        //     }
        // }

        // newTriangles = new int[sideLength * sideLength * 6];

        // for (int x = 0; x < sideLength; x++) {
        //     for (int z = 0; z < sideLength; z++) {
        //         int s = x * vertSide + z;
        //         int i = (x * sideLength + z) * 6;
        //         newTriangles[i] = s;
        //         newTriangles[i + 1] = s + 1;
        //         newTriangles[i + 2] = s + vertSide;
        //         newTriangles[i + 3] = s + vertSide + 1;
        //         newTriangles[i + 4] = s + vertSide;
        //         newTriangles[i + 5] = s + 1;
        //     }
        // }


        // gameObject.AddComponent<MeshFilter>();
        // gameObject.AddComponent<MeshRenderer>();
        // Mesh mesh = new Mesh();
        // GetComponent<MeshFilter>().mesh = mesh;
        // GetComponent<MeshRenderer>().material = m_Material;
        // mesh.Clear();


        // mesh.SetIndexBufferParams(newVertices.Length, UnityEngine.Rendering.IndexFormat.UInt32);
        // mesh.vertices = newVertices;
        // mesh.uv = newUV;
        // mesh.triangles = newTriangles;
    }
}
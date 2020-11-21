using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using Unity.Collections.LowLevel.Unsafe;
using System.Runtime.InteropServices;
using System;

using UnityEngine.Rendering;
using Unity.Collections;

public class MyArray : MonoBehaviour
{
    [DllImport("meshgen")]
    private static extern IntPtr get_array(UIntPtr length);
    [DllImport("meshgen")]
    private static extern IntPtr hello_world();
    [DllImport("meshgen")]
    private static extern int fill_array(IntPtr buf, int vCnt);

    [System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
    struct ExampleVertex
    {
        public Vector3 pos;
        public Vector3 normal;
        public Vector3 tangent;
        public Vector2 uv;
    }
    
    // Start is called before the first frame update
    void Start()
    {
        var vertexCount = 500000000;
        var verts = new NativeArray<int>(vertexCount, Allocator.Temp);
        // fill vertex buffer
        unsafe {
            vertexCount = fill_array(new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(verts)), vertexCount);
            // foreach (var item in verts)
            // {
            //     Debug.Log(item);
            // }
        }


        unsafe {
            // int* array = (int*)get_array((UIntPtr)10);
            // // OK
            // for (int i = 0; i < 10; i++) {
            //     Debug.Log(array[i]);
            //     continue;
            // }
            // OK, but I'd like to avoid Marshal above
            // Debug.Log(Marshal.PtrToStringAnsi(hello_world()));
        }

    }
}

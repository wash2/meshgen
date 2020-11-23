using UnityEngine;
using System.Collections;
using Unity.Collections.LowLevel.Unsafe;
using Unity.Collections;
using System.Runtime.InteropServices;
using System;

public class MapGenerator : MonoBehaviour {

    Texture2D texture;

	public int mapWidth;
	public int mapHeight;
    public uint seed = 0;
    [Range(0,8)]
    public uint octaves = 4;
	public double noiseScale;
    [Range(0,1)]
    public double persistence = .5;
    [Range(0,1)]
    public double bias_gain_a = .5;
    [Range(0,8)]
    public double lacunarity = .5;
    public double displacement = 1.5;
    public Vector2 offset = new Vector2();
    public CustomGradient colorGradients;

    IntPtr texturegen; 
	public bool autoUpdate;
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

    void Start()
    {

        texturegen = get_mountainous_terrain_texturegen((UIntPtr)mapWidth, (UIntPtr)mapHeight, seed, octaves, noiseScale, persistence, lacunarity, displacement, bias_gain_a);
        Debug.Log(texturegen);

        GenerateMap();
    }

    void OnApplicationQuit()
    {
        free_mountainous_terrain_texturegen(texturegen);
        Debug.Log("freed texturegen");
    }

	public void GenerateMap() {
        Texture2D texture = new Texture2D (mapWidth, mapHeight);

        // update dim
        set_mountainous_terrain_texturegen_dim(texturegen, (UIntPtr)mapWidth, (UIntPtr)mapHeight);
        set_mountainous_terrain_texturegen_noise(texturegen, seed, octaves, noiseScale, persistence, lacunarity, displacement, bias_gain_a);
        var data = texture.GetRawTextureData<Color32>();
        unsafe {
            fixed(Vector2* offser_ptr = &offset) {
                fill_mountainous_terrain_texture_2d(texturegen, new IntPtr(NativeArrayUnsafeUtility.GetUnsafePtr(data)), new IntPtr(offser_ptr));
            }
        }
		MapDisplay display = FindObjectOfType<MapDisplay> ();

		display.DrawNoiseMap (texture);
	}

    void OnValidate() {
        if (mapWidth < 1) {
            mapWidth = 1;
        }
        if (mapHeight < 1) {
            mapHeight = 1;
        }
        if (lacunarity < 1) {
            lacunarity = 1;
        }
        if (octaves < 0) {
            octaves = 0;
        }
    }
}
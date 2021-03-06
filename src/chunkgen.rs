extern crate more_asserts;

use std::{convert::TryInto, mem, panic, ptr};
use log::{info, error};
use rayon::prelude::*;

use crate::{MountainousTerrainNoise, Noise2D, unity::SimpleVertex, gradient::{BlendType, ColorKey, ColorKeyGradient}, unity::Color32, unity::{Normal32, Position2D32, Position3D32, Quad, Tangent32, TexCoord32, Triangle, Vertex}};

pub trait ChunkGen2D {
    fn get(&self, pos: Position2D32) -> f64;
    fn get_side_len(&self) -> usize;
    fn get_height(&self) -> f64;
    fn get_color_gradient(&self) -> &ColorKeyGradient;

    fn fill_chunk_2d(&self, v_buffer: &mut [SimpleVertex], indx_buffer: &mut [Quad], tex_buffer: &mut [Color32], plane_pos: Position3D32) where Self: Sync {
        info!("filling chunk with data");

        let half_side_len: f32 = self.get_side_len() as f32 / 2f32;
        let vert_side: i32 = (self.get_side_len() + 1).try_into().unwrap();
        let height = self.get_height();
        let color_gradient = self.get_color_gradient();

        v_buffer.par_iter_mut().zip(tex_buffer.par_iter_mut()).enumerate().for_each(|(i, (cur_v, cur_pix))| {
            let x_pos = -half_side_len + (i as i32 % vert_side) as f32;
            let z_pos = -half_side_len + (i as i32 / vert_side) as f32;
            let cur_pos = Position2D32{ x: x_pos, y: z_pos } + Position2D32::from(plane_pos);
            let n = self.get(cur_pos);
            *cur_pix = color_gradient.get_color(n);
            cur_v.pos = Position3D32{
                x: x_pos,
                y: (n * height) as f32 + plane_pos.y,
                z: z_pos
            };
            // cur_v.norm = Normal32{
            //     x: (0f32),
            //     y: (1f32),
            //     z: (0f32)
            // };
            // cur_v.tangent = Tangent32{
            //     w: (1f32),
            //     x: (1f32),
            //     y: (0f32),
            //     z: (0f32)
            // };
            cur_v.uv = TexCoord32{
                u: ((i as i32 % vert_side) as f32 / vert_side as f32),
                v: ((i as i32 / vert_side) as f32 / vert_side as f32), 
            };
        });

        indx_buffer.par_iter_mut().enumerate().for_each(|(i, cur_tri)| {
            let z = (i / self.get_side_len()) as i32;
            let x = (i % self.get_side_len()) as i32;
            let s = x * vert_side + z;
            cur_tri.v1 = s + vert_side;
            cur_tri.v2 = s + 1;
            cur_tri.v3 = s;
            cur_tri.v4 = s + 1;
            cur_tri.v5 = s + vert_side;
            cur_tri.v6 = s + vert_side + 1;
        });
    }
}

#[derive(Clone, Debug)]
pub struct MountainousTerrainChunkGen {
    pub side_len: usize,
    pub height: f64,
    pub noise: MountainousTerrainNoise,
    pub color_gradient: ColorKeyGradient,
}

impl MountainousTerrainChunkGen {
    pub fn build(side_len: usize, height: f64) -> Self {
        Self {
            side_len,
            height,
            noise: MountainousTerrainNoise::default(),
            color_gradient: ColorKeyGradient::default()
        }
    }

    fn to_ptr(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    fn free(ptr: *mut Self) {
        if !ptr.is_null() {
            // SHOULD BE DROPPED AUTOMATICALLY
            let _mynoise: Box<Self> = unsafe { Box::from_raw(ptr) };
        }
    }
}

impl ChunkGen2D for MountainousTerrainChunkGen {
    fn get(&self, pos: Position2D32) -> f64 {
        self.noise.get(pos)
    }

    fn get_side_len(&self) -> usize {
        self.side_len
    }

    fn get_height(&self) -> f64 {
        self.height
    }

    fn get_color_gradient(&self) -> &ColorKeyGradient {
        &self.color_gradient
    }
}

impl Default for MountainousTerrainChunkGen {
    fn default() -> Self {
        MountainousTerrainChunkGen::build(100, 50.0)
    }
}

#[no_mangle]
pub extern "C" fn fill_mountainous_terrain_chunk(chunkgen: *mut MountainousTerrainChunkGen, vert_buf: *mut SimpleVertex, indx_buf: *mut Quad, tex_buf: *mut Color32, plane_pos: *mut Position3D32) -> *const u8 {
    if chunkgen.is_null() {
        "ERROR: pointer to texturegen is null\0".as_ptr()
    }
    else if vert_buf.is_null() {
        "ERROR: pointer to vert_buf is null\0".as_ptr()
    }
    else if indx_buf.is_null() {
        "ERROR: pointer to indx_buf is null\0".as_ptr()
    }
    else if tex_buf.is_null() {
        "ERROR: pointer to tex_buf is null\0".as_ptr()
    }
    else if plane_pos.is_null() {
        "ERROR: pointer to plane_pos is null\0".as_ptr()
    }
    else {
        let res = panic::catch_unwind(|| {
            unsafe {
                let chunkgen = Box::from_raw(chunkgen);
                let v_count =  (chunkgen.side_len + 1) * (chunkgen.side_len + 1);
                let quad_count = chunkgen.side_len * chunkgen.side_len;
                let vert_buffer: &mut [SimpleVertex] = std::slice::from_raw_parts_mut(vert_buf, v_count);
                let tex_buffer: &mut [Color32] = std::slice::from_raw_parts_mut(tex_buf, v_count);
                let indx_buffer: &mut [Quad] = std::slice::from_raw_parts_mut(indx_buf, quad_count);
                let plane_pos = *plane_pos;
                chunkgen.fill_chunk_2d(vert_buffer, indx_buffer, tex_buffer, plane_pos);
                Box::leak(chunkgen);
            }
        });
        match res {
            Ok(_) => "OK\0".as_ptr(),
            Err(err) => format!("{:?}\0", err).as_ptr()
        }
    }  
}

#[no_mangle]
pub extern "C" fn free_mountainous_terrain_chunkgen(mut _ptr: *mut MountainousTerrainChunkGen) {
    info!("freeing mountainous terrain chunkgen");
    if !_ptr.is_null() {
        MountainousTerrainChunkGen::free(_ptr);
    } 
    _ptr = ptr::null_mut() as *mut MountainousTerrainChunkGen;
}

#[no_mangle]
pub extern "C" fn get_mountainous_terrain_chunkgen(side_len: usize, height: f64) -> *mut  MountainousTerrainChunkGen {
    info!("getting mountainous terrain chunkgen");
    MountainousTerrainChunkGen::build(side_len, height).to_ptr()
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_chunkgen_dim(chunkgen: *mut MountainousTerrainChunkGen, side_len: usize, height: f64) {
    info!("setting mountainous terrain chunkgen dimensions");
    if !chunkgen.is_null() {
        let mut chunkgen = unsafe { Box::from_raw(chunkgen) };
        chunkgen.side_len = side_len;
        chunkgen.height = height;
        Box::leak(chunkgen);
    } 
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_chunkgen_noise(chunkgen: *mut MountainousTerrainChunkGen, seed: u32, octaves: u32, scale: f64, persistance: f64, lacunarity: f64, displacement: f64, a: f64, bezier_bias_from: *mut Position2D32, bezier_bias_to: *mut Position2D32, bezier_bias_corner_curvature: f64) {
    info!("setting mountainous terrain chunkgen noise");
    if !chunkgen.is_null() {
        let mut chunkgen = unsafe { Box::from_raw(chunkgen) };
        unsafe {
            chunkgen.noise = MountainousTerrainNoise::build(seed, scale, persistance, lacunarity, octaves, displacement, a, *bezier_bias_from, *bezier_bias_to, bezier_bias_corner_curvature);
        }
        Box::leak(chunkgen);
    } 
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_chunkgen_color_gradient(chunkgen: *mut MountainousTerrainChunkGen, color_key_arr: *mut ColorKey, key_cnt: usize, blend_mode: bool) {
    if !chunkgen.is_null() {
        let mut chunkgen = unsafe { Box::from_raw(chunkgen) };
        let blend_type: BlendType = match blend_mode {
            true => BlendType::Linear,
            _ => BlendType::Discrete,
        };
        if key_cnt < 1 {
            chunkgen.color_gradient = ColorKeyGradient{blend_type, keys: Vec::new()};
        }
        else {
            let res = panic::catch_unwind(|| {
                unsafe {
                    let keys: &mut [ColorKey] = std::slice::from_raw_parts_mut(color_key_arr, key_cnt);
                    info!("{:?}", keys);
                    keys
                }
            });
            chunkgen.color_gradient = match res {
                Ok(keys) => ColorKeyGradient{blend_type, keys: (Vec::from(keys))},
                Err(_) => {
                    error!("failed to get colorkey gradient from pointer. using default");
                    ColorKeyGradient{blend_type, keys: Vec::new()}
                }
            }
        }
        Box::leak(chunkgen);
    } 
}

#[no_mangle]
pub extern "C" fn get_mountainous_terrain_chunk_geometry_desc(chunkgen: *mut MountainousTerrainChunkGen, v_count: ptr::NonNull<i32>, e_count: ptr::NonNull<i32>, f_count: ptr::NonNull<i32>) -> *const u8 {
    let chunkgen = unsafe { Box::from_raw(chunkgen) };
    let side_len = chunkgen.side_len;
    Box::leak(chunkgen);
    let _v_count = ((side_len + 1) * (side_len + 1)) as u64;
    let _e_count = (2 * side_len + 3 * side_len * side_len) as u64;
    let _f_count = (2 * side_len * side_len) as u64;

    let _v_buf_bytes = _v_count * mem::size_of::<SimpleVertex>() as u64;
    let _e_buf_bytes = _e_count * mem::size_of::<i32>() as u64;
    let _f_buf_bytes = _f_count * mem::size_of::<Triangle>() as u64;
    
    if _e_buf_bytes >= i32::MAX as u64 {
        format!("Edge list would require too many bytes! {}\0",  _e_buf_bytes).as_ptr()
    }
    else if _v_buf_bytes >= i32::MAX as u64 {
        format!("Vertex list would require too many bytes! {}\0",  _v_buf_bytes).as_ptr()
    }
    else if _f_buf_bytes >= i32::MAX as u64 {
        format!("Face list would require too many bytes! {}\0",  _f_buf_bytes).as_ptr()
    }
    else {
        unsafe {
            let v_count_mut = v_count.as_ptr();
            *v_count_mut = _v_count.try_into().unwrap();
            let e_count_mut = e_count.as_ptr();
            *e_count_mut = _e_count.try_into().unwrap();
            let f_count_mut = f_count.as_ptr();
            *f_count_mut = _f_count.try_into().unwrap();
        }
        "OK\0".as_ptr()
    }
}


#[cfg(test)]
mod chunk_tests {
    // use crate::unity::{Quad, Vertex, Position3D32};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

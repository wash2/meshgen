extern crate more_asserts;

use crate::{gradient::{BlendType, ColorKey, ColorKeyGradient}, noisegen::{MountainousTerrainNoise, Noise2D}, unity::{Position2D32, Color32}};
use std::{panic};
use std::ptr;
use log::info;
use rayon::{prelude::*};

pub trait TextureGen2D {
    fn get(&self, pos: Position2D32) -> f64;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_color_gradient(&self) -> ColorKeyGradient;
    fn fill_texture_2d(&self, tex_buffer: &mut [Color32], pos: Position2D32) where Self: Sync { 
        // TODO: idk why this would fail, but if it can, it should match the Result of the possible failure and return a good error message
        let width = self.get_width();
        let height = self.get_height();
        let color_gradient = self.get_color_gradient();
        let half_width = width as f32 / 2.0;
        let half_height = height as f32 / 2.0;

        tex_buffer.par_iter_mut().enumerate().for_each(|(i, cur_pix)| {
            let cur_pos = Position2D32{x: (i % width) as f32 - half_width, y: (i / height) as f32 -  half_height} + pos;
            let h = color_gradient.get_color(self.get(cur_pos) / 2_f64 + 0.5_f64);
            *cur_pix = h;
        });
    }
}

#[derive(Clone, Debug)]
pub struct MountainousTerrainTextureGen {
    pub width: usize,
    pub height: usize,
    pub noise: MountainousTerrainNoise,
    pub color_gradient: ColorKeyGradient,
}

impl MountainousTerrainTextureGen {
    pub fn build(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            noise: MountainousTerrainNoise::default(),
            color_gradient: ColorKeyGradient::default(),
        }
    }

    fn to_ptr(self) -> *mut MountainousTerrainTextureGen {
        Box::into_raw(Box::new(self))
    }

    fn free(ptr: *mut MountainousTerrainTextureGen) {
        if !ptr.is_null() {
            // SHOULD BE DROPPED AUTOMATICALLY
            let _mynoise: Box<MountainousTerrainTextureGen> = unsafe { Box::from_raw(ptr) };
        }
    }
}

impl Default for MountainousTerrainTextureGen {
    fn default() -> Self {
        MountainousTerrainTextureGen::build(100, 100)
    }
}

impl TextureGen2D for MountainousTerrainTextureGen {
    fn get(&self, pos: Position2D32) -> f64 {
        self.noise.get(pos)
    }
    fn get_width(&self) -> usize {
        self.width
    }
    fn get_height(&self) -> usize {
        self.height
    }

    fn get_color_gradient(&self) -> ColorKeyGradient {
        self.color_gradient.clone()
    }
}

#[no_mangle]
pub extern "C" fn free_mountainous_terrain_texturegen(mut _ptr: *mut MountainousTerrainTextureGen) {
    if !_ptr.is_null() {
        MountainousTerrainTextureGen::free(_ptr);
    } 
    _ptr = ptr::null_mut() as *mut MountainousTerrainTextureGen;
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_texturegen_dim(texturegen: *mut MountainousTerrainTextureGen, width: usize, height: usize) {
    if !texturegen.is_null() {
        let mut texturegen = unsafe { Box::from_raw(texturegen) };
        texturegen.width = width;
        texturegen.height = height;
        Box::leak(texturegen);
    } 
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_texturegen_noise(texturegen: *mut MountainousTerrainTextureGen, seed: u32, octaves: u32, scale: f64, persistance: f64, lacunarity: f64, displacement: f64, a: f64, bezier_bias_from: Position2D32, bezier_bias_to: Position2D32, bezier_bias_corner_curvature: f64) {
    if !texturegen.is_null() {
        let mut texturegen = unsafe { Box::from_raw(texturegen) };
        texturegen.noise = MountainousTerrainNoise::build(seed, scale, persistance, lacunarity, octaves, displacement, a, bezier_bias_from, bezier_bias_to, bezier_bias_corner_curvature);
        Box::leak(texturegen);
    } 
}

#[no_mangle]
pub extern "C" fn set_mountainous_terrain_texturegen_color_gradient(texturegen: *mut MountainousTerrainTextureGen, color_key_arr: *mut ColorKey, key_cnt: usize, blend_mode: bool) {
    if !texturegen.is_null() {
        let mut texturegen = unsafe { Box::from_raw(texturegen) };
        let blend_type: BlendType = match blend_mode {
            true => BlendType::Linear,
            _ => BlendType::Discrete,
        };
        if key_cnt < 1 {
            texturegen.color_gradient = ColorKeyGradient{blend_type, keys: Vec::new()};
        }
        else {
            let res = panic::catch_unwind(|| {
                unsafe {
                    let keys: &mut [ColorKey] = std::slice::from_raw_parts_mut(color_key_arr, key_cnt);
                    info!("{:?}", keys);
                    keys
                }
            });
            texturegen.color_gradient = match res {
                Ok(keys) => ColorKeyGradient{blend_type, keys: (Vec::from(keys))},
                Err(_) => ColorKeyGradient{blend_type, keys: Vec::new()}, // TODO set error message here
            }
        }
        Box::leak(texturegen);
    } 
}

#[no_mangle]
pub extern "C" fn get_mountainous_terrain_texturegen() -> *mut  MountainousTerrainTextureGen {
    info!("getting MountainousTerrainTextureGen...");
    MountainousTerrainTextureGen::default().to_ptr()
}

#[no_mangle]
pub extern "C" fn fill_mountainous_terrain_texture_2d(texturegen: *mut MountainousTerrainTextureGen, bufptr: *mut Color32, pos: *mut Position2D32) -> *const u8 {
    // info!("filling texture buffer...");
    if texturegen.is_null() {
        "ERROR: pointer to texturegen is null\0".as_ptr()
    }
    else if bufptr.is_null() {
        "ERROR: pointer to bufptr is null\0".as_ptr()
    }
    else if pos.is_null() {
        "ERROR: pointer to pos is null\0".as_ptr()
    }
    else {
        let res = panic::catch_unwind(|| {
            unsafe {
                let texturegen = Box::from_raw(texturegen);
                // info!("{:?}", texturegen);
                let pix_cnt = texturegen.width * texturegen.height;
                let tx_buffer: &mut [Color32] = std::slice::from_raw_parts_mut(bufptr, pix_cnt);
                let pos = *pos;
                // info!("{:?}", pos);
                texturegen.fill_texture_2d(tx_buffer, pos);
                Box::leak(texturegen);
            }
        });
        match res {
            Ok(_) => "OK\0".as_ptr(),
            Err(err) => format!("{:?}\0", err).as_ptr()
        }
    }    
}

#[cfg(test)]
mod texture_tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

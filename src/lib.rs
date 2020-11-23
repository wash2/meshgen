extern crate more_asserts;
#[path = "noisegen.rs"]
mod noisegen;
#[path = "unity.rs"]
mod unity;
#[path = "chunkgen.rs"]
mod chunkgen;
#[path = "texturegen.rs"]
mod texturegen;
#[path = "gradient.rs"]
mod gradient;

// re-export module ffi
pub use chunkgen::{get_mountainous_terrain_chunkgen, free_mountainous_terrain_chunkgen, fill_mountainous_terrain_chunk};
pub use texturegen::{get_mountainous_terrain_texturegen, free_mountainous_terrain_texturegen, fill_mountainous_terrain_texture_2d};
use noisegen::*;

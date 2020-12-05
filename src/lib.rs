extern crate more_asserts;
extern crate simplelog;


use log::{info, error};
use simplelog::*;
use std::fs::File;
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

#[no_mangle]
pub extern "C" fn init_logger() {
    match CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create(r"C:\Users\ashtr\meshgen\meshgen.log").unwrap()),
        ]
    ) {
        Ok(_) => info!("logger initialized"),
        Err(_) => error!("logger already initialized")
    }
}

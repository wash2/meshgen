use noise::{NoiseFn, Seedable};
use crate::unity::Position2D32;
use rand::{Rng, SeedableRng};
use rand::rngs::{SmallRng};

struct Flat {}
impl<T> NoiseFn<T> for Flat {
    fn get(&self, _point: T) -> f64 {0f64}
}

pub trait Noise2D<'a> {
    fn get(&self, pos: Position2D32) -> f64;
}

#[derive(Clone, Default, Debug)]
pub struct MountainousTerrainNoise {
    pub(crate) noise: noise::Perlin,
    amplitude: Vec<f64>,
    frequency: Vec<f64>,
    offsets: Vec<[f64; 2]>,
    max_noise_sum: f64,
    pub(crate) scale: f64,
    pub(crate) octaves: u32,
    pub(crate) seed: u32,
    pub(crate) persistance: f64,
    pub(crate) lacunarity: f64,
}


impl MountainousTerrainNoise {
    pub fn build(seed: u32, scale: f64, persistance: f64, lacunarity: f64, octaves: u32) -> Self {
        let mut small_rng = SmallRng::seed_from_u64(seed as u64);

        let noise = noise::Perlin::new();
        noise.set_seed(small_rng.gen());

        Self {
            noise,
            seed,
            scale,
            octaves,
            persistance,
            lacunarity,
            amplitude: (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).collect(),
            frequency: (0..octaves).map(|i| { f64::powf(lacunarity, i.into()) }).collect(),
            max_noise_sum: (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).sum(),
            offsets: (0..octaves).map(|_i| { 
                [
                    small_rng.gen_range(-100000.0, 100000.0), 
                    small_rng.gen_range(-100000.0, 100000.0)
                ]}).collect()
        }
    }
}

impl<'a> Noise2D<'_> for MountainousTerrainNoise {
    fn get(&self, pos: Position2D32) -> f64 {
        let my_sum: f64 = (0..self.octaves)
            .map(|i: u32| -> f64 {
                let sample_x: f64 = pos.x as f64 / self.scale * self.frequency[i as usize] + self.offsets[i as usize][0];
                let sample_y: f64 = pos.y as f64 / self.scale * self.frequency[i as usize] + self.offsets[i  as usize][1];
                (self.noise.get([sample_x, sample_y]) + 1.0)  * self.amplitude[i as usize] / 2.0
            })
            .sum();
        my_sum / self.max_noise_sum
    }
}

#[cfg(test)]
mod noise_tests {
    use more_asserts::{assert_ge, assert_le};
    use noise::NoiseFn;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_perlin_magnitude() {
        let scale = 10;
        let height = 1000;
        let width = 1000;
        let noise = noise::Perlin::new();
        let octaves = 4;
        let persistance = 1.0;
        let amplitude: Vec<f64> = (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).collect();
        let projected_max: f64 = (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).sum();
        let mut max = 0_f64;
        let mut min = 0_f64;
        for p in 0..(width * height) {
            let h: f64 = (0..octaves)
            .map(|i: u32| -> f64 {
                (noise.get([(p % width) as f64 / scale as f64, (p / height) as f64 / scale as f64]) + 1.0)  * amplitude[i as usize] / 2.0
            })
            .sum();
            // let h = noise.get([(i % width) as f64 / scale as f64, (i / height) as f64 / scale as f64]);
            max = f64::max(h, max);
            min = f64::min(h, min);
        }
        println!("{:.2}", max);
        println!("{:.2}", min);
        println!("{:.2}", projected_max);
    }
}

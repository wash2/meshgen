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
    fn get_bias(&self, h: f64, a: f64) -> f64;
    fn get_gain(&self, h: f64) -> f64;
}

#[derive(Clone, Default, Debug)]
pub struct MountainousTerrainNoise {
    pub(crate) noise: noise::Perlin,
    amplitude: Vec<f64>,
    frequency: Vec<f64>,
    offsets: Vec<[f64; 2]>,
    max_noise_sum: f64,
    displacement: f64,
    pub(crate) scale: f64,
    pub(crate) octaves: u32,
    pub(crate) seed: u32,
    pub(crate) persistance: f64,
    pub(crate) lacunarity: f64,
    pub(crate) a: f64,
}


impl MountainousTerrainNoise {
    pub fn build(seed: u32, scale: f64, persistance: f64, lacunarity: f64, octaves: u32, displacement: f64, a: f64) -> Self {
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
            offsets: (0..octaves + 1).map(|_i| { 
                [
                    small_rng.gen_range(-100000.0, 100000.0), 
                    small_rng.gen_range(-100000.0, 100000.0)
                ]}).collect(),
            displacement,
            a,
        }
    }
}

impl<'a> Noise2D<'_> for MountainousTerrainNoise {
    fn get(&self, pos: Position2D32) -> f64 {
        let my_sum: f64 = (0..self.octaves)
            .map(|i: u32| -> f64 {
                let mut sample_x: f64 = pos.x as f64 / self.scale * self.frequency[i as usize] + self.offsets[i as usize][0];
                let mut sample_y: f64 = pos.y as f64 / self.scale * self.frequency[i as usize] + self.offsets[i  as usize][1];
                if self.displacement > 0.0 {
                    sample_x += self.displacement * self.noise.get([self.offsets[self.octaves as usize][0] + sample_x, self.offsets[self.octaves as usize][1]]);
                    sample_y += self.displacement * self.noise.get([self.offsets[self.octaves as usize][0], self.offsets[self.octaves as usize][1] + sample_y]);
                }
                ( ( self.noise.get([sample_x, sample_y]) + 1.0 ) / 2.0 )  * self.amplitude[i as usize]
            })
            .sum();
        self.get_gain(my_sum / self.max_noise_sum)
    }

    fn get_bias(&self, h: f64, a: f64) -> f64 {
        h / ( (1.0 / a - 2.0) * (1.0 - h) + 1.0 )
    }

    fn get_gain(&self, h: f64) -> f64 {
        if h < 0.5 {
            self.get_bias(2.0 * h, self.a) / 2.0
        }
        else {
            ( self.get_bias(2.0 * h - 1.0, 1.0 - self.a) + 1.0 ) / 2.0
        }
    }
}

#[cfg(test)]
mod noise_tests {
    use more_asserts::{assert_ge, assert_le};
    use noise::NoiseFn;

    use crate::unity::Position2D32;

    use super::{MountainousTerrainNoise, Noise2D};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_perlin_magnitude() {
        let scale = 10.0;
        let height = 1000;
        let width = 100;
        let noise = noise::Perlin::new();
        let octaves = 1;
        let persistance = 0.1;
        let amplitude: Vec<f64> = (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).collect();
        let projected_max: f64 = (0..octaves).map(|i| { f64::powf(persistance, i.into()) }).sum();
        let a = 0.9;
        let my_noise = MountainousTerrainNoise::build(0, scale, persistance, 0.5, 3, -1.9, 0.2);
        let mut max = 0_f64;
        let mut min = 1_f64;
        let mut sum = 0_f64;
        let mut count_low_5 = 0;
        for p in 0..(width * height) {
            let h = my_noise.get(Position2D32{x: (p % width) as f32, y: (p / width) as f32});           
            // let h = noise.get([(i % width) as f64 / scale as f64, (i / height) as f64 / scale as f64]);
            max = f64::max(h, max);
            min = f64::min(h, min);
            sum += h;
            if h < 0.05 {
                count_low_5 +=1;
            }
        }
        println!("{:.5}", count_low_5 as f64 / (width * height) as f64);
        println!("{:.2}", sum / (width * height) as f64);
        println!("{:.2}", max);
        println!("{:.2}", min);
        println!("{:.2}", projected_max);
    }
}

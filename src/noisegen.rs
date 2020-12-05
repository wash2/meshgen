use log::info;
use noise::{NoiseFn, Seedable};
use crate::unity::{Position2D32, Lerp};
use rand::{Rng, SeedableRng};
use rand::rngs::{SmallRng};
use lyon_geom::CubicBezierSegment;

struct Flat {}
impl<T> NoiseFn<T> for Flat {
    fn get(&self, _point: T) -> f64 {0f64}
}

pub trait Noise2D<'a> {
    fn get(&self, pos: Position2D32) -> f64 {
        let scale = self.get_scale();
        let frequency = self.get_frequency();
        let octaves = self.get_octaves();
        let offsets = self.get_offsets();
        let displacement = self.get_displacement();
        let amplitude = self.get_amplitude();

        let my_sum: f64 = (0..octaves)
            .map(|i: u32| -> f64 {
                let mut sample_x: f64 = pos.x as f64 / scale * frequency[i as usize] + offsets[i as usize][0];
                let mut sample_y: f64 = pos.y as f64 / scale * frequency[i as usize] + offsets[i  as usize][1];
                if displacement > 0.0 {
                    sample_x += displacement * self.get_noise([offsets[octaves as usize][0] + sample_x, offsets[octaves as usize][1] + sample_y]);
                    sample_y += displacement * self.get_noise([offsets[(octaves + 1) as usize][0] + sample_x, offsets[(octaves + 1) as usize][1] + sample_y]);
                }
                ( ( self.get_noise([sample_x, sample_y]) + 1.0 ) / 2.0 )  * amplitude[i as usize]
            })
            .sum();
            let h_pre = my_sum / self.get_max_noise_sum();
            let h = self.get_gain(self.get_bezier_bias(h_pre));
            // info!("{:?}", (h_pre,h));
            h
    }
    fn get_noise(&self, pos: [f64; 2]) -> f64;
    fn get_displacement_noise(&self, pos: [f64; 2]) -> f64;
    fn get_scale(&self) -> f64;
    fn get_offsets(&self) -> &Vec<[f64; 2]>;
    fn get_frequency(&self) -> &Vec<f64>;
    fn get_amplitude(&self) -> &Vec<f64>;
    fn get_displacement(&self) -> f64;
    fn get_bias(&self, h: f64, a: f64) -> f64;
    fn get_gain(&self, h: f64) -> f64;
    fn get_bezier_bias(&self, h:f64) -> f64;
    fn get_max_noise_sum(&self) -> f64;
    fn get_octaves(&self) -> u32;
    fn get_fast_bias_gain_control_param(&self) -> f64;
}

#[derive(Clone, Debug)]
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
    pub(crate) fast_bias_gain_control_param: f64,
    pub(crate) bezier_bias: CubicBezierSegment<f32>,
}


impl MountainousTerrainNoise {
    pub fn build(seed: u32, scale: f64, persistance: f64, lacunarity: f64, octaves: u32, displacement: f64, a: f64, bezier_bias_from: Position2D32, bezier_bias_to: Position2D32, bezier_bias_corner_curvature: f64) -> Self {
        let mut small_rng = SmallRng::seed_from_u64(seed as u64);

        let noise = noise::Perlin::new();
        noise.set_seed(small_rng.gen());
        
        
        let m_from = bezier_bias_from.y / bezier_bias_from.x;
        let m_to = (1.0 - bezier_bias_to.y) / (1.0 - bezier_bias_to.x);
        let c_x = (-m_to * bezier_bias_to.x  + bezier_bias_to.y) / (m_from - m_to);
        let c_y = m_from * c_x;

        let ctrl = Position2D32{x: c_x, y: c_y};

        let ctrl1 = bezier_bias_from.lerp_bounded(ctrl, bezier_bias_corner_curvature);
        let ctrl2 = bezier_bias_to.lerp_bounded(ctrl, bezier_bias_corner_curvature);

        info!("{:?}", CubicBezierSegment{from: bezier_bias_from.into(), to: bezier_bias_to.into(), ctrl1: ctrl1.into(), ctrl2: ctrl2.into()});
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
            offsets: (0..octaves + 2).map(|_i| { 
                [
                    small_rng.gen_range(-100000.0, 100000.0), 
                    small_rng.gen_range(-100000.0, 100000.0)
                ]}).collect(),
            displacement,
            fast_bias_gain_control_param: a,
            bezier_bias: CubicBezierSegment{from: bezier_bias_from.into(), to: bezier_bias_to.into(), ctrl1: ctrl1.into(), ctrl2: ctrl2.into()}
        }
    }
}

impl Default for MountainousTerrainNoise {
    fn default() -> Self {
        MountainousTerrainNoise::build(
            0,
            50.0,
            0.5,
            0.2,
            3,
            -1.0,
            0.3,
            Position2D32{x: 0.4, y: 0.0},
            Position2D32{x: 0.5, y:0.1},
            0.5
        )
    }
}

impl<'a> Noise2D<'_> for MountainousTerrainNoise {
    fn get_bias(&self, h: f64, a: f64) -> f64 {
        h / ( (1.0 / a - 2.0) * (1.0 - h) + 1.0 )
    }

    fn get_gain(&self, h: f64) -> f64 {
        if h < 0.5 {
            self.get_bias(2.0 * h, self.fast_bias_gain_control_param) / 2.0
        }
        else {
            ( self.get_bias(2.0 * h - 1.0, 1.0 - self.fast_bias_gain_control_param) + 1.0 ) / 2.0
        }
    }

    fn get_noise(&self, pos: [f64; 2]) -> f64 {
        self.noise.get(pos)
    }

    fn get_displacement_noise(&self, pos: [f64; 2]) -> f64 {
        self.noise.get(pos)
    }

    fn get_scale(&self) -> f64 {
        self.scale
    }

    fn get_offsets(&self) -> &Vec<[f64; 2]> {
        &self.offsets
    }

    fn get_frequency(&self) -> &Vec<f64> {
        &self.frequency
    }

    fn get_displacement(&self) -> f64 {
        self.displacement
    }

    fn get_amplitude(&self) -> &Vec<f64> {
        &self.amplitude
    }

    fn get_max_noise_sum(&self) -> f64 {
        self.max_noise_sum
    }

    fn get_octaves(&self) -> u32 {
        self.octaves
    }

    fn get_fast_bias_gain_control_param(&self) -> f64 {
        self.fast_bias_gain_control_param
    }

    fn get_bezier_bias(&self, h:f64) -> f64 {
        if h < self.bezier_bias.from.x.into() {
            h * self.bezier_bias.from.y as f64 / self.bezier_bias.from.x as f64
        }
        else if h > self.bezier_bias.to.x.into() {
            (h - self.bezier_bias.to.x as f64 ) * (1.0 - self.bezier_bias.to.y as f64) / (1.0 - self.bezier_bias.to.x as f64) + self.bezier_bias.to.y as f64
        }
        else {
            self.bezier_bias.y((h as f32 - self.bezier_bias.from.x) / (self.bezier_bias.to.x - self.bezier_bias.from.x)).into()
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
        let my_noise = MountainousTerrainNoise::default();
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

    #[test]
    fn test_bezier_bias() {

        let my_noise = MountainousTerrainNoise::default();
        let my_range = 0..21;
        let bias_out: Vec<(f64, f64)> = my_range.map(|i| {(i as f64 / 20.0, my_noise.get_bezier_bias(i as f64 / 20.0))}).collect();
        println!("{:?}", bias_out);
    }
}

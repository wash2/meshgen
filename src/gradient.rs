use crate::unity::{Lerp, Color32};

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct ColorKey {
    pub color: Color32,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct ColorKeyMessage
{
    color_keys: *mut ColorKey,
    key_cnt: usize,
    blend_mode: bool,
}
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum BlendType {
    Discrete,
    Linear,
}

#[repr(C)]
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct ColorKeyGradient {
    pub keys: Vec<ColorKey>,
    pub blend_type: BlendType
}

impl ColorKeyGradient {
    pub fn get_color(&self, t: f64) -> Color32 {
        if self.keys.len() > 0 {
            let mut l = self.keys[0];
            let mut r = self.keys[self.keys.len() - 1];
            let mut adjusted_t = t;
            for i in 0..self.keys.len() - 1 {
                if self.keys[i].t < t as f32 {
                    l = self.keys[i];
                    r = self.keys[i + 1];
                    adjusted_t = ((t as f32 - l.t) / (r.t - l.t)).into();
                }
            }
            match self.blend_type {
                BlendType::Discrete => r.color,
                BlendType::Linear => l.color.lerp_bounded(r.color, adjusted_t)
            }
        }
        else {
            let black = Color32{r: 0, g: 0, b: 0, a: 255};
            let white = Color32{r: 255, g: 255, b: 255, a: 255};
            black.lerp_bounded(white, t)
        }
    }
}

impl Default for ColorKeyGradient {
    fn default() -> Self {
        ColorKeyGradient{ blend_type: BlendType::Discrete, keys: Vec::new() }
    }
}

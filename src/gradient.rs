use crate::unity::Color32;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct ColorKey {
    pub color: Color32,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[repr(C)]
struct ColorKeyMessage
{
    color_keys: *mut ColorKey,
    key_cnt: usize,
    blend_mode: bool,
}

pub enum BlendType {
    Discrete,
    Linear,
}

#[repr(C)]
pub struct Gradient {
    pub keys: Vec<ColorKey>,
    pub blend_type: BlendType
}

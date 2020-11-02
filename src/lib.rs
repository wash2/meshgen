use std::ptr;
use std::f64;
use rayon::prelude::*;
use half::f16;

#[repr(C)]
struct Position32 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
struct Normal16 {
    x: f16,
    y: f16,
    z: f16,
}

#[repr(C)]
struct Normal32 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
struct Tangent32 {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
struct TangentU8 {
    w: u8,
    x: u8,
    y: u8,
    z: u8,
}

#[repr(C)]
pub struct Vertex {
    pos: Position32,
    norm: Normal32,
    tangent: Tangent32,
}

#[no_mangle]
pub extern "C" fn get_array(length: usize) -> *mut i32 {
    let mut test = vec![68; length];
    let ptr = test.as_mut_ptr();

    std::mem::forget(test); // so that it is not destructed at the end of the scope

    ptr
}

#[no_mangle]
pub extern "C" fn hello_world() -> *const u8 {
    "Hello, world!\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn fill_array(buf: ptr::NonNull<i32>, len: usize) -> usize {
    let v_buffer: &mut [i32] = unsafe { std::slice::from_raw_parts_mut(buf.as_ptr(), len) };
    v_buffer.par_iter_mut().enumerate().for_each(|(i, elem)| *elem = i as i32);
    len
}

// future goals: generate different wrapper functions for each type permutation and make get_plane generic...
//  Vertex<f32, half::f16 | f32, u8 | f32>
#[no_mangle]
pub extern "C" fn get_plane(buf: ptr::NonNull<Vertex>, len: usize) -> usize {
    let side_len =  (len as f64).sqrt() as usize;
    let v_count =  side_len * side_len;
    let v_buffer: &mut [Vertex] = unsafe { std::slice::from_raw_parts_mut(buf.as_ptr(), v_count) };

    let half_side_len = side_len as f32 / 2f32;

    v_buffer.par_iter_mut().enumerate().for_each(|(i, cur_v)| {
        cur_v.pos = Position32{
            x: (-half_side_len + (i % side_len) as f32),
            y: (0f32),
            z: (-half_side_len + (i / side_len) as f32)
        };
        cur_v.norm = Normal32{
            x: (0f32), 
            y: (1f32), 
            z: (0f32)
        };
        cur_v.tangent = Tangent32{
            w: (0f32),
            x: (0f32), 
            y: (0f32), 
            z: (0f32)
        }
    });

    v_count
}

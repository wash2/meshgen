use std::{convert::TryInto, convert::TryFrom, ptr, mem};
use rayon::{prelude::*};
use half::f16;
use noise;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Position32 {
    x: f32,
    y: f32,
    z: f32,
}
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Normal16 {
    x: f16,
    y: f16,
    z: f16,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Normal32 {
    x: f32,
    y: f32,
    z: f32,
}
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Tangent32 {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct TangentU8 {
    w: u8,
    x: u8,
    y: u8,
    z: u8,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct TexCoord32 {
    u: f32,
    v: f32
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Vertex {
    pos: Position32,
    norm: Normal32,
    tangent: Tangent32,
    uv: TexCoord32,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Triangle {
    v1: i32,
    v2: i32,
    v3: i32
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Quad {
    v1: i32,
    v2: i32,
    v3: i32,
    v4: i32,
    v5: i32,
    v6: i32,
}

#[repr(C)]
enum PlaneType
{
    Flat,
    FBM,
    Worley,
}

impl TryFrom<i32> for PlaneType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == PlaneType::Flat as i32 => Ok(PlaneType::Flat),
            x if x == PlaneType::FBM as i32 => Ok(PlaneType::FBM),
            x if x == PlaneType::Worley as i32 => Ok(PlaneType::Worley),
            _ => Err(()),
        }
    }
}

// future goals: generate different wrapper functions for each type permutation and make get_plane generic...
//  Vertex<f32, half::f16 | f32, u8 | f32>
#[no_mangle]
pub extern "C" fn fill_plane(vert_buf: ptr::NonNull<Vertex>, indx_buf: ptr::NonNull<Quad>, side_len: usize, plane_type: i32) -> *const u8 {
    match plane_type.try_into() {
        Ok(plane_type) => {
            let v_count =  (side_len + 1) * (side_len + 1);
            let tri_count = side_len * side_len;
            let v_buffer: &mut [Vertex] = unsafe { std::slice::from_raw_parts_mut(vert_buf.as_ptr(), v_count) };
            let indx_buffer: &mut [Quad] = unsafe { std::slice::from_raw_parts_mut(indx_buf.as_ptr(), tri_count) };

            fill_plane_buffers(v_buffer, indx_buffer, side_len, plane_type);

            "OK\0".as_ptr()
        },
        Err(_) => "Invalid plane type requested.\0".as_ptr()
    }
}

fn fill_plane_buffers(v_buffer: &mut [Vertex], indx_buffer: &mut [Quad], side_len: usize, plane_type: PlaneType) {
    let half_side_len = side_len as f32 / 2f32;
    let vert_side: i32 = (side_len + 1).try_into().unwrap();

    v_buffer.par_iter_mut().enumerate().for_each(|(i, cur_v)| {
        
        cur_v.pos = Position32{
            x: (-half_side_len + (i as i32 % vert_side) as f32),
            y: (0f32),
            z: (-half_side_len + (i as i32 / vert_side) as f32)
        };
        cur_v.norm = Normal32{
            x: (0f32), 
            y: (1f32), 
            z: (0f32)
        };
        cur_v.tangent = Tangent32{
            w: (1f32),
            x: (1f32), 
            y: (0f32), 
            z: (0f32)
        };
        cur_v.uv = TexCoord32{
            u: ((i as i32 % vert_side) as f32 / vert_side as f32),
            v: ((i as i32 / vert_side) as f32 / vert_side as f32), 
        };
    });

    indx_buffer.par_iter_mut().enumerate().for_each(|(i, cur_tri)| {
        let z = (i / side_len) as i32;
        let x = (i % side_len) as i32;
        let s = x * vert_side + z;
        cur_tri.v1 = s + vert_side;
        cur_tri.v2 = s + 1;
        cur_tri.v3 = s;
        cur_tri.v4 = s + 1;
        cur_tri.v5 = s + vert_side;
        cur_tri.v6 = s + vert_side + 1;
    });
}

#[no_mangle]
pub extern "C" fn get_plane_desc(side_len: usize, v_count: ptr::NonNull<i32>, e_count: ptr::NonNull<i32>, f_count: ptr::NonNull<i32>) -> *const u8 {
    let _v_count = ((side_len + 1) * (side_len + 1)) as u64;
    let _e_count = (2 * side_len + 3 * side_len * side_len) as u64;
    let _f_count = (2 * side_len * side_len) as u64;

    let _v_buf_bytes = _v_count * mem::size_of::<Vertex>() as u64;
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

#[no_mangle]
pub extern "C" fn get_plane(vptr: *mut *const Vertex, iptr: *mut *const Quad, side_len: usize) -> usize {
    let v_count =  (side_len + 1) * (side_len + 1);
    let quad_count = side_len * side_len;

    let mut v_buffer: Vec<Vertex> = vec![Default::default(); v_count];
    let mut indx_buffer: Vec<Quad> = vec![Default::default(); quad_count];

    fill_plane_buffers(v_buffer.as_mut_slice(), indx_buffer.as_mut_slice(), side_len, PlaneType::Flat);

    unsafe {
        *vptr = v_buffer.as_ptr();
        *iptr = indx_buffer.as_ptr();
    }
    std::mem::forget(v_buffer); // so that it is not destructed at the end of the scope
    std::mem::forget(indx_buffer); // so that it is not destructed at the end of the scope

    0
}

#[test]
fn test_square_plane_one() {
    let side_len = 2;
    let v_count =  (side_len + 1) * (side_len + 1);
    let quad_count = side_len * side_len;

    let mut v_buffer: Vec<Vertex> = vec![Default::default(); v_count];
    let mut indx_buffer: Vec<Quad> = vec![Default::default(); quad_count];

    fill_plane_buffers(v_buffer.as_mut_slice(), indx_buffer.as_mut_slice(), side_len, PlaneType::Flat);   
}

#[test]
fn test_square_plane_two() {
    let side_len = 2;
    let v_count =  (side_len + 1) * (side_len + 1);
    let quad_count = side_len * side_len;

    let mut v_buffer: Vec<Vertex> = vec![Default::default(); v_count];
    let mut indx_buffer: Vec<Quad> = vec![Default::default(); quad_count];

    fill_plane_buffers(v_buffer.as_mut_slice(), indx_buffer.as_mut_slice(), side_len, PlaneType::Flat);   
}

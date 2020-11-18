use meshgen;

fn main() {
    let height = 100;
    let width = 100;
    let scale = 10;

    let mut tex_buffer: Vec<meshgen::Color32> = vec![Default::default(); width * height];

    meshgen::fill_texture_buffer_2d(tex_buffer.as_mut_slice(), width, height, scale as f32, meshgen::PlaneType::Worley);   
    for pix in tex_buffer {

        println!("{:?}", pix);
    }
}
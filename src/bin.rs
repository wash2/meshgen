// use meshgen;

fn main() {
    println!("hi");
    let scale = 10;
    let height = 1000;
    let width = 1000;
    let noise = noise::Perlin::new();

    let mut max = 0_f64;
    let mut max = 0_f64;
    for i in 0..(width * height) {
        let h = noise.get([(i % width) as f64 / scale as f64, (i / height) as f64 / scale as f64]);
        max = f64::max(h, max);
        min = f64::min(h, min);
    }
    println!(max);
    println!(min);
}
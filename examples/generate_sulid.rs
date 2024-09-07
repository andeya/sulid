use sulid::SulidGenerator;

fn main() {
    let generator = SulidGenerator::new(1, 1);

    for _ in 0..5 {
        let id = generator.generate();
        println!("SULID: {}", id);
    }
}

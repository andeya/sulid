use sulid::SulidGenerator;

fn main() {
    let generator = SulidGenerator::v1_new(1, 1);

    for _ in 0..3 {
        let id = generator.generate();
        println!("SULID-V1: {}", id);
    }

    let generator = SulidGenerator::v2_new(1);

    for _ in 0..3 {
        let id = generator.generate();
        println!("SULID-V2: {}", id);
    }
}

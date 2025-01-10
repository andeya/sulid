use sulid::{SulidGenerator, TimestampType};

fn main() {
    let generator = SulidGenerator::new1(1, 1, TimestampType::MS);

    for _ in 0..3 {
        let id = generator.generate();
        println!("SULID-V1: {}", id);
    }

    let generator = SulidGenerator::new2(1, TimestampType::MS);

    for _ in 0..3 {
        let id = generator.generate();
        println!("SULID-V2: {}", id);
    }
}

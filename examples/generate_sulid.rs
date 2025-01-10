use sulid::{SulidGenerator, TimestampType};

fn main() {
    #[cfg(feature = "std")]
    {
        let generator = SulidGenerator::new1(1, 1, TimestampType::MS);

        for _ in 0..3 {
            let id = generator.generate();
            println!("SULID-MS: {}", id);
        }

        let generator = SulidGenerator::new2(1, TimestampType::US);

        for _ in 0..3 {
            let id = generator.generate();
            println!("SULID-US: {}", id);
        }
    }
    #[cfg(not(feature = "std"))]
    {
        let generator = SulidGenerator::new1(1, 1, TimestampType::MS);

        for i in 0..3 {
            let id = generator.generate(1736611200000, i);
            println!("SULID-MS: {}", id);
        }

        let generator = SulidGenerator::new2(1, TimestampType::US);

        for i in 0..3 {
            let id = generator.generate(1736611200000, i);
            println!("SULID-US: {}", id);
        }
    }
}

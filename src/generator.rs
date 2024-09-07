#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(not(feature = "std"))]
pub use self::no_std_feature::*;
#[cfg(feature = "std")]
pub use self::std_feature::*;

mod no_std_feature {
    use crate::Sulid;

    /// A struct for generating Snowflake-inspired ULIDs (SULIDs).
    /// This generator combines the benefits of ULID and Snowflake to
    /// ensure unique, lexicographically sortable identifiers across multiple
    /// data centers and machines.
    pub struct SulidGenerator {
        /// The ID of the data center (5 bits).
        pub(super) data_center_id: u8,
        /// The ID of the machine within the data center (5 bits).
        pub(super) machine_id: u8,
    }

    impl SulidGenerator {
        /// Creates a new SulidGenerator.
        ///
        /// # Arguments
        ///
        /// * `data_center_id` - A 5-bit identifier for the data center (0-31).
        /// * `machine_id` - A 5-bit identifier for the machine within the data center (0-31).
        ///
        /// # Panics
        ///
        /// Panics if `data_center_id` or `machine_id` is outside the 0-31 range.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::new(1, 1);
        /// ```
        pub fn new(data_center_id: u8, machine_id: u8) -> Self {
            // Ensure the data_center_id and machine_id are within the 5-bit range.
            assert!(
                data_center_id < 32,
                "data_center_id must be in the range 0-31"
            );
            assert!(machine_id < 32, "machine_id must be in the range 0-31");
            SulidGenerator {
                data_center_id,
                machine_id,
            }
        }

        /// Generates a new SULID.
        ///
        /// This method generates a 128-bit unique identifier that combines
        /// a timestamp, data center ID, machine ID, and a random component.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::new(1, 1);
        /// let sulid = generator.generate(1, 1);
        /// println!("Generated SULID: {}", sulid);
        /// ```
        #[cfg(not(feature = "std"))]
        pub fn generate(&self, timestamp_ms: u64, random: u128) -> Sulid {
            Sulid::from_parts(timestamp_ms, random, self.data_center_id, self.machine_id)
        }
    }

    #[cfg(not(feature = "std"))]
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::new(1, 1);

            let id1 = generator.generate(1, 1);
            let id2 = generator.generate(2, 2);

            assert_ne!(id1, id2);
        }

        #[test]
        #[should_panic(expected = "data_center_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range data_center_id panics.
        fn data_center_id_out_of_range() {
            let _ = SulidGenerator::new(32, 1);
        }

        #[test]
        #[should_panic(expected = "machine_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range machine_id panics.
        fn machine_id_out_of_range() {
            let _ = SulidGenerator::new(1, 32);
        }
    }
}

#[cfg(feature = "std")]
mod std_feature {
    use super::no_std_feature::SulidGenerator as InnerSulidGenerator;
    use crate::Sulid;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::sync::Mutex;
    use std::time::SystemTime;

    /// A struct for generating Snowflake-inspired ULIDs (SULIDs).
    /// This generator combines the benefits of ULID and Snowflake to
    /// ensure unique, lexicographically sortable identifiers across multiple
    /// data centers and machines.
    pub struct SulidGenerator {
        inner: InnerSulidGenerator,
        /// The random number generator wrapped in a mutex for thread safety.
        rng: Mutex<StdRng>,
    }

    impl SulidGenerator {
        /// Creates a new SulidGenerator.
        ///
        /// # Arguments
        ///
        /// * `data_center_id` - A 5-bit identifier for the data center (0-31).
        /// * `machine_id` - A 5-bit identifier for the machine within the data center (0-31).
        ///
        /// # Panics
        ///
        /// Panics if `data_center_id` or `machine_id` is outside the 0-31 range.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::new(1, 1);
        /// ```
        pub fn new(data_center_id: u8, machine_id: u8) -> Self {
            let inner = InnerSulidGenerator::new(data_center_id, machine_id);
            let rng = Mutex::new(StdRng::from_entropy());
            SulidGenerator { inner, rng }
        }

        /// Generates a new SULID.
        ///
        /// This method generates a 128-bit unique identifier that combines
        /// a timestamp, data center ID, machine ID, and a random component.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::new(1, 1);
        /// let sulid = generator.generate();
        /// println!("Generated SULID: {}", sulid);
        /// ```
        #[inline]
        pub fn generate(&self) -> Sulid {
            let mut rng = self.rng.lock().unwrap();
            Sulid::from_datetime_with_source(
                SystemTime::now(),
                &mut *rng,
                self.inner.data_center_id,
                self.inner.machine_id,
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::new(1, 1);

            let id1 = generator.generate();
            let id2 = generator.generate();

            assert_ne!(id1, id2);
        }

        #[test]
        #[should_panic(expected = "data_center_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range data_center_id panics.
        fn data_center_id_out_of_range() {
            let _ = SulidGenerator::new(32, 1);
        }

        #[test]
        #[should_panic(expected = "machine_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range machine_id panics.
        fn machine_id_out_of_range() {
            let _ = SulidGenerator::new(1, 32);
        }
    }
}

#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(not(feature = "std"))]
pub use self::no_std_feature::*;
#[cfg(feature = "std")]
pub use self::std_feature::*;

mod no_std_feature {
    use crate::Sulid;

    pub(super) enum Version {
        V1 {
            /// The ID of the data center (5 bits).
            data_center_id: u8,
            /// The ID of the machine within the data center (5 bits).
            machine_id: u8,
        },
        V2 {
            /// The ID of the combination of data_center_id and machine_id.
            worker_id: u16,
        },
    }

    /// A struct for generating Snowflake-inspired ULIDs (SULIDs).
    /// This generator combines the benefits of ULID and Snowflake to
    /// ensure unique, lexicographically sortable identifiers across multiple
    /// data centers and machines.
    pub struct SulidGenerator(pub(super) Version);

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
        /// let generator = SulidGenerator::v1_new(1, 1);
        /// ```
        pub fn v1_new(data_center_id: u8, machine_id: u8) -> Self {
            // Ensure the data_center_id and machine_id are within the 5-bit range.
            assert!(
                data_center_id < 32,
                "data_center_id must be in the range 0-31"
            );
            assert!(machine_id < 32, "machine_id must be in the range 0-31");
            SulidGenerator(Version::V1 {
                data_center_id,
                machine_id,
            })
        }

        /// Creates a new SulidGenerator.
        ///
        /// # Arguments
        ///
        /// * `worker_id` - A 10-bit identifier combining data_center_id and machine_id (range: 0-1023).
        ///
        /// # Panics
        ///
        /// Panics if `worker_id` is outside the 0-1023 range.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::v2_new(1);
        /// ```
        pub fn v2_new(worker_id: u16) -> Self {
            // Ensure the worker_id is within the 10-bit range.
            assert!(worker_id < 32, "worker_id must be in the range 0-1023");
            SulidGenerator(Version::V2 { worker_id })
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
        /// let generator = SulidGenerator::v1_new(1, 1);
        /// let sulid = generator.generate(1, 1);
        /// println!("Generated SULID-V1: {}", sulid);
        ///
        /// let generator = SulidGenerator::v2_new(1);
        /// let sulid = generator.generate(1, 1);
        /// println!("Generated SULID-V2: {}", sulid);
        /// ```
        #[cfg(not(feature = "std"))]
        pub fn generate(&self, timestamp_ms: u64, random: u128) -> Sulid {
            match self.0 {
                Version::V1 {
                    data_center_id,
                    machine_id,
                } => Sulid::v1_from_parts(timestamp_ms, random, data_center_id, machine_id),
                Version::V2 { worker_id } => Sulid::v2_from_parts(timestamp_ms, random, worker_id),
            }
        }
    }

    #[cfg(not(feature = "std"))]
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::v1_new(1, 1);

            let id1 = generator.generate(1, 1);
            let id2 = generator.generate(2, 2);

            assert_ne!(id1, id2);

            let generator = SulidGenerator::v2_new(1);

            let id1 = generator.generate(1, 1);
            let id2 = generator.generate(2, 2);

            assert_ne!(id1, id2);
        }

        #[test]
        #[should_panic(expected = "data_center_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range data_center_id panics.
        fn v1_data_center_id_out_of_range() {
            let _ = SulidGenerator::v1_new(32, 1);
        }

        #[test]
        #[should_panic(expected = "machine_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range machine_id panics.
        fn v1_machine_id_out_of_range() {
            let _ = SulidGenerator::v1_new(1, 32);
        }

        #[test]
        #[should_panic(expected = "worker_id must be in the range 0-1023")]
        /// Test that creating a SulidGenerator with an out-of-range worker_id panics.
        fn v2_worker_id_out_of_range() {
            let _ = SulidGenerator::v2_new(1024);
        }
    }
}

#[cfg(feature = "std")]
mod std_feature {
    use super::no_std_feature::{SulidGenerator as InnerSulidGenerator, Version};
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
        /// let generator = SulidGenerator::v1_new(1, 1);
        /// ```
        pub fn v1_new(data_center_id: u8, machine_id: u8) -> Self {
            let inner = InnerSulidGenerator::v1_new(data_center_id, machine_id);
            let rng = Mutex::new(StdRng::from_entropy());
            SulidGenerator { inner, rng }
        }

        /// Creates a new SulidGenerator.
        ///
        /// # Arguments
        ///
        /// * `worker_id` - A 10-bit identifier combining data_center_id and machine_id (range: 0-1023).
        ///
        /// # Panics
        ///
        /// Panics if `worker_id` is outside the 0-1023 range.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::SulidGenerator;
        /// let generator = SulidGenerator::v2_new(1);
        /// ```
        pub fn v2_new(worker_id: u16) -> Self {
            let inner = InnerSulidGenerator::v2_new(worker_id);
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
        /// let generator = SulidGenerator::v1_new(1, 1);
        /// let sulid = generator.generate();
        /// println!("Generated SULID 1: {}", sulid);
        /// let generator = SulidGenerator::v2_new(1);
        /// let sulid = generator.generate();
        /// println!("Generated SULID 2: {}", sulid);
        /// ```
        #[inline]
        pub fn generate(&self) -> Sulid {
            let mut rng = self.rng.lock().unwrap();
            match self.inner.0 {
                Version::V1 {
                    data_center_id,
                    machine_id,
                } => Sulid::v1_from_datetime_with_source(
                    SystemTime::now(),
                    &mut *rng,
                    data_center_id,
                    machine_id,
                ),
                Version::V2 { worker_id } => {
                    Sulid::v2_from_datetime_with_source(SystemTime::now(), &mut *rng, worker_id)
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::v1_new(1, 1);

            let id1 = generator.generate();
            let id2 = generator.generate();

            assert_ne!(id1, id2);

            let generator = SulidGenerator::v2_new(1);

            let id1 = generator.generate();
            let id2 = generator.generate();

            assert_ne!(id1, id2);
        }
    }
}

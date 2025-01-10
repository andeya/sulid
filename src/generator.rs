#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(not(feature = "std"))]
pub use self::no_std_feature::*;
#[cfg(feature = "std")]
pub use self::std_feature::*;

mod no_std_feature {
    use crate::{Sulid, TimestampType, WorkerId};

    /// A struct for generating Snowflake-inspired ULIDs (SULIDs).
    /// This generator combines the benefits of ULID and Snowflake to
    /// ensure unique, lexicographically sortable identifiers across multiple
    /// data centers and machines.
    pub struct SulidGenerator(pub(crate) WorkerId, pub(crate) TimestampType);

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
        /// ```rust
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new1(1, 1, TimestampType::MS);
        /// ```
        pub fn new1(data_center_id: u8, machine_id: u8, ts_type: TimestampType) -> Self {
            // Ensure the data_center_id and machine_id are within the 5-bit range.
            assert!(
                data_center_id < 32,
                "data_center_id must be in the range 0-31"
            );
            assert!(machine_id < 32, "machine_id must be in the range 0-31");
            SulidGenerator(
                WorkerId::Two {
                    data_center_id,
                    machine_id,
                },
                ts_type,
            )
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
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new2(1, TimestampType::MS);
        /// ```
        pub fn new2(worker_id: u16, ts_type: TimestampType) -> Self {
            // Ensure the worker_id is within the 10-bit range.
            assert!(worker_id < 32, "worker_id must be in the range 0-1023");
            SulidGenerator(WorkerId::One(worker_id), ts_type)
        }

        /// Generates a new SULID.
        ///
        /// This method generates a 128-bit unique identifier that combines
        /// a timestamp, data center ID, machine ID, and a random component.
        ///
        /// # Example
        ///
        /// ```
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new1(1, 1, TimestampType::MS);
        /// let sulid = generator.generate(1, 1);
        /// println!("Generated SULID-V1: {}", sulid);
        ///
        /// let generator = SulidGenerator::new2(1, TimestampType::MS);
        /// let sulid = generator.generate(1, 1);
        /// println!("Generated SULID-V2: {}", sulid);
        /// ```
        #[cfg(not(feature = "std"))]
        pub fn generate(&self, timestamp: u64, random: u128) -> Sulid {
            Sulid::from_parts(self.1.new_ts_u64(timestamp), random, self.0)
        }
    }

    #[cfg(not(feature = "std"))]
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::new1(1, 1, TimestampType::MS);

            let id1 = generator.generate(1, 1);
            let id2 = generator.generate(2, 2);

            assert_ne!(id1, id2);

            let generator = SulidGenerator::new2(1, TimestampType::MS);

            let id1 = generator.generate(1, 1);
            let id2 = generator.generate(2, 2);

            assert_ne!(id1, id2);
        }

        #[test]
        #[should_panic(expected = "data_center_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range data_center_id panics.
        fn data_center_id_out_of_range() {
            let _ = SulidGenerator::new1(32, 1, TimestampType::MS);
        }

        #[test]
        #[should_panic(expected = "machine_id must be in the range 0-31")]
        /// Test that creating a SulidGenerator with an out-of-range machine_id panics.
        fn machine_id_out_of_range() {
            let _ = SulidGenerator::new1(1, 32, TimestampType::MS);
        }

        #[test]
        #[should_panic(expected = "worker_id must be in the range 0-1023")]
        /// Test that creating a SulidGenerator with an out-of-range worker_id panics.
        fn worker_id_out_of_range() {
            let _ = SulidGenerator::new2(1024, TimestampType::MS);
        }
    }
}

#[cfg(feature = "std")]
mod std_feature {
    use super::no_std_feature::SulidGenerator as InnerSulidGenerator;
    use crate::{Sulid, TimestampType};
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
        /// ```rust
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new1(1, 1, TimestampType::MS);
        /// ```
        pub fn new1(data_center_id: u8, machine_id: u8, ts_type: TimestampType) -> Self {
            let inner = InnerSulidGenerator::new1(data_center_id, machine_id, ts_type);
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
        /// ```rust
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new2(1, TimestampType::MS);
        /// ```
        pub fn new2(worker_id: u16, ts_type: TimestampType) -> Self {
            let inner = InnerSulidGenerator::new2(worker_id, ts_type);
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
        /// use sulid::{SulidGenerator, TimestampType};
        /// let generator = SulidGenerator::new1(1, 1, TimestampType::MS);
        /// let sulid = generator.generate();
        /// println!("Generated SULID 1: {}", sulid);
        /// let generator = SulidGenerator::new2(1, TimestampType::MS);
        /// let sulid = generator.generate();
        /// println!("Generated SULID 2: {}", sulid);
        /// ```
        #[inline]
        pub fn generate(&self) -> Sulid {
            let mut rng = self.rng.lock().unwrap();
            Sulid::from_datetime_source(SystemTime::now(), &mut *rng, self.inner.0, self.inner.1)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        /// Test that two generated SULIDs are unique.
        fn generate_unique_ids() {
            let generator = SulidGenerator::new1(1, 1, TimestampType::MS);

            let id1 = generator.generate();
            let id2 = generator.generate();

            assert_ne!(id1, id2);

            let generator = SulidGenerator::new2(1, TimestampType::MS);

            let id1 = generator.generate();
            let id2 = generator.generate();

            assert_ne!(id1, id2);
        }
    }
}

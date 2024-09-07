#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
//! # SULID: Snowflake-inspired Universally Unique Lexicographically Sortable Identifier
//!
//! SULID is a unique ID generation algorithm that combines the benefits of ULID and Snowflake.
//! It offers a highly efficient, reliable, and lexicographically sortable identifier, which ensures
//! uniqueness across multiple data centers and machines, making it ideal for high-concurrency
//! distributed environments.
//!
//! ## Features
//!
//! - **High Concurrency Support**: Efficiently generates unique IDs in high-concurrency environments.
//! - **Time Ordered**: Retains the time-ordered characteristic of ULID.
//! - **Distributed Uniqueness**: Ensures unique IDs across distributed environments by incorporating
//!   data center and machine IDs.
//! - **Readability**: Produces shorter, human-readable identifiers.
//!
//! ## Overview
//!
//! SULID is based on the ULID (Universally Unique Lexicographically Sortable Identifier) format but
//! incorporates additional bits for a data center ID and machine ID, similar to Snowflake. This design
//! ensures uniqueness in distributed environments and maintains time-ordering characteristics, making
//! it suitable for applications requiring both high concurrency and global uniqueness, such as
//! microservice architectures, log tracking, and order generation.
//!
//! ## ID Format
//!
//! SULIDs have a unique structure comprising the following parts, adding up to a 128-bit identifier:
//!
//! ### Version1
//! 1. **Timestamp**: 48 bits, representing the epoch time in milliseconds.
//! 2. **Random Number**: 70 bits of randomness to ensure uniqueness within the same millisecond.
//! 3. **Data Center ID**: 5 bits, identifying the data center.
//! 4. **Machine ID**: 5 bits, identifying the machine within the data center.
//!
//! Here is a visual breakdown of the SULID format:
//!
//! ```
//! | 48-bit Timestamp | 70-bit Random Number | 5-bit Data Center ID | 5-bit Machine ID |
//! ```
//!
//! ### Version2
//! 1. **Timestamp**: 48 bits, representing the epoch time in milliseconds.
//! 2. **Random Number**: 70 bits of randomness to ensure uniqueness within the same millisecond.
//! 3. **Worker ID**: 10 bits, the combination of data_center_id and machine_id.
//!
//! Here is a visual breakdown of the SULID format:
//!
//! ```
//! | 48-bit Timestamp | 70-bit Random Number | 10-bit Worker ID |
//! ```
//!
//! ## Usage
//!
//! Add the following dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sulid = "0.6"
//! ```
//!
//! Then, you can generate SULIDs as follows:
//!
//! ```rust
//! use sulid::SulidGenerator;
//!
//! fn main() {
//!     let generator = SulidGenerator::v1_new(1, 1);
//!
//!     for _ in 0..3 {
//!         #[cfg(feature = "std")]
//!         let id = generator.generate();
//!         #[cfg(not(feature = "std"))]
//!         let id = generator.generate(1, 1);
//!         println!("SULID-V1: {}", id);
//!     }
//!
//!     let generator = SulidGenerator::v2_new(1);
//!
//!     for _ in 0..3 {
//!         #[cfg(feature = "std")]
//!         let id = generator.generate();
//!         #[cfg(not(feature = "std"))]
//!         let id = generator.generate(1, 1);
//!         println!("SULID-V2: {}", id);
//!     }
//! }
//! ```

pub use generator::SulidGenerator;
pub use sulid::Sulid;
// Republic ULID
pub use ulid;
pub use ulid::{DecodeError, EncodeError, ULID_LEN};

mod generator;
pub(crate) mod sulid;

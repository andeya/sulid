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
//! 1. **Timestamp**: 48 bits, representing the epoch time in milliseconds.
//! 2. **Data Center ID**: 5 bits, identifying the data center.
//! 3. **Machine ID**: 5 bits, identifying the machine within the data center.
//! 4. **Random Number**: 70 bits of randomness to ensure uniqueness within the same millisecond.
//!
//! Here is a visual breakdown of the SULID format:
//!
//! ```
//! | 48-bit Timestamp | 5-bit Data Center ID | 5-bit Machine ID | 70-bit Random Number |
//! ```
//!
//! ## Usage
//!
//! Add the following dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sulid = "0.1.0"
//! ```
//!
//! Then, you can generate SULIDs as follows:
//!
//! ```rust
//! use sulid::SulidGenerator;
//!
//! fn main() {
//!     let generator = SulidGenerator::new(1, 1);
//!
//!     for _ in 0..5 {
//!         #[cfg(feature = "std")]
//!         let id = generator.generate();
//!         #[cfg(not(feature = "std"))]
//!         let id = generator.generate(1, 1);
//!         println!("SULID: {}", id);
//!     }
//! }
//! ```

pub use generator::SulidGenerator;
pub use sulid::Sulid;
pub use ulid::{DecodeError, EncodeError, ULID_LEN};
mod generator;
pub(crate) mod sulid;

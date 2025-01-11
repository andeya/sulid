# SULID: Snowflake-inspired Universally Unique Lexicographically Sortable Identifier

SULID is a unique ID generation algorithm that combines the benefits of ULID and Snowflake. It offers a highly efficient, reliable, and lexicographically sortable identifier, which ensures uniqueness across multiple data centers and machines, making it ideal for high-concurrency distributed environments.

[![Crates.io](https://img.shields.io/crates/v/sulid)](https://crates.io/crates/sulid)
[![Documentation](https://shields.io/docsrs/sulid)](https://docs.rs/sulid)
[![License](https://img.shields.io/crates/l/sulid)](https://github.com/andeya/sulid?tab=MIT-1-ov-file)


## Features

- **High Concurrency Support**: Efficiently generates unique IDs in high-concurrency environments.
- **Time Ordered**: It retains the time-ordered characteristic of ULID and supports both millisecond-level ordering and microsecond-level ordering.
- **Distributed Uniqueness**: Ensures unique IDs across distributed environments by incorporating data center and machine IDs.
- **Readability**: Produces shorter, human-readable identifiers.

## Overview

SULID is based on the ULID (Universally Unique Lexicographically Sortable Identifier) format but incorporates additional bits for a data center ID and machine ID, similar to Snowflake. This design ensures uniqueness in distributed environments and maintains time-ordering characteristics, making it suitable for applications requiring both high concurrency and global uniqueness, such as microservice architectures, log tracking, and order generation.

## ID Format

SULIDs have a unique structure comprising the following parts, adding up to a 128-bit identifier:

### Millisecond-level Ordering
1. **Timestamp**: 42 bits, representing the epoch time in milliseconds.
2. **Random Number**: 76 bits of randomness to ensure uniqueness within the same millisecond.
3. **Worker ID**: 10 bits, the combination of data_center_id and machine_id.
   - **Data Center ID**: 5 bits, identifying the data center.
   - **Machine ID**: 5 bits, identifying the machine within the data center.

Here is a visual breakdown of the SULID format:

```
| 42-bit Timestamp | 76-bit Random Number | 10-bit Worker ID (5-bit Data Center ID | 5-bit Machine ID) |
```

### Microsecond-level Ordering
1. **Timestamp**: 52 bits, representing the epoch time in milliseconds.
2. **Random Number**: 66 bits of randomness to ensure uniqueness within the same millisecond.
3. **Worker ID**: 10 bits, the combination of data_center_id and machine_id.
   - **Data Center ID**: 5 bits, identifying the data center.
   - **Machine ID**: 5 bits, identifying the machine within the data center.

Here is a visual breakdown of the SULID format:

```
| 52-bit Timestamp | 66-bit Random Number | 10-bit Worker ID (5-bit Data Center ID | 5-bit Machine ID) |
```


## Installation

To use SULID, add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
sulid = "0.7"
```

## Usage

Here's how you can use the `SulidGenerator` in your project:

```rust
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
```

## Example Output

Running the example code generates SULIDs such as:

```
SULID-MS: 352KWKVJQ81KMWGE5946JR1T11
SULID-MS: 352KWKVJQ0X87EE0GF7CJRSX11
SULID-MS: 352KWKVJPDTEAM8Q606NK6CC11
SULID-US: 352KWKVJP01ZBHJAYYFQZ08A01
SULID-US: 352KWKVJP011FSHCDGJFW75G01
SULID-US: 352KWKVJP010NZCGVHKM373901
```

## Benefits

- High Concurrency Support: The algorithm is designed to generate IDs efficiently in environments with high concurrency, making it suitable for distributed systems.
- Time Ordered: SULID retains the time-ordering characteristic of ULID. This feature is beneficial for logging systems and event sourcing where chronological order is essential.
- Distributed Uniqueness: By incorporating data center and machine IDs similar to Snowflake's approach, SULID ensures IDs are unique across different machines and data centers.
- Readability: Compared to traditional UUIDs, SULID produces shorter and more human-readable identifiers, making them easier to work with in certain scenarios.
- Traceability: The time-ordered nature of SULID makes it easier to trace and debug events in distributed systems.

## How It Works

SULID leverages both ULID and Snowflake's strengths:
- `ULID`: Provides lexicographically sortable identifiers based on a timestamp and randomness.
- `Snowflake`: Adds data center and machine IDs to ensure distributed uniqueness.

By combining these two approaches, SULID generates IDs that are globally unique, time-ordered, and suitable for high-concurrency distributed environments.

## License

This project is licensed under the MIT License.

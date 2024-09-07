# SULID: Snowflake-inspired Universally Unique Lexicographically Sortable Identifier

SULID is a unique ID generation algorithm that combines the benefits of ULID and Snowflake. It offers a highly efficient, reliable, and lexicographically sortable identifier, which ensures uniqueness across multiple data centers and machines, making it ideal for high-concurrency distributed environments.

[![Crates.io](https://img.shields.io/crates/v/sulid)](https://crates.io/crates/sulid)
[![Documentation](https://shields.io/docsrs/sulid)](https://docs.rs/sulid)
[![License](https://img.shields.io/crates/l/sulid)](https://github.com/andeya/sulid?tab=MIT-1-ov-file)


## Features

- **High Concurrency Support**: Efficiently generates unique IDs in high-concurrency environments.
- **Time Ordered**: Retains the time-ordered characteristic of ULID.
- **Distributed Uniqueness**: Ensures unique IDs across distributed environments by incorporating data center and machine IDs.
- **Readability**: Produces shorter, human-readable identifiers.

## Overview

SULID is based on the ULID (Universally Unique Lexicographically Sortable Identifier) format but incorporates additional bits for a data center ID and machine ID, similar to Snowflake. This design ensures uniqueness in distributed environments and maintains time-ordering characteristics, making it suitable for applications requiring both high concurrency and global uniqueness, such as microservice architectures, log tracking, and order generation.

## ID Format

SULIDs have a unique structure comprising the following parts, adding up to a 128-bit identifier:

### Version1
1. **Timestamp**: 48 bits, representing the epoch time in milliseconds.
2. **Random Number**: 70 bits of randomness to ensure uniqueness within the same millisecond.
3. **Data Center ID**: 5 bits, identifying the data center.
4. **Machine ID**: 5 bits, identifying the machine within the data center.

Here is a visual breakdown of the SULID format:

```
| 48-bit Timestamp | 70-bit Random Number | 5-bit Data Center ID | 5-bit Machine ID |
```

### Version2
1. **Timestamp**: 48 bits, representing the epoch time in milliseconds.
2. **Random Number**: 70 bits of randomness to ensure uniqueness within the same millisecond.
3. **Worker ID**: 10 bits, the combination of data_center_id and machine_id.

Here is a visual breakdown of the SULID format:

```
| 48-bit Timestamp | 70-bit Random Number | 10-bit Worker ID |
```


## Installation

To use SULID, add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
sulid = "0.5"
```

## Usage

Here's how you can use the `SulidGenerator` in your project:

```rust
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
```

## Example Output

Running the example code generates SULIDs such as:

```
SULID-V1: 01J75Y6K09Q0VD4D7HCF496K11
SULID-V1: 01J75Y6K09ZGD0ATFGNJ7TWZ11
SULID-V1: 01J75Y6K09DWG2RPNEXNYSRP11
SULID-V2: 01J75Y6K09FVT8QTQVYGSDJ401
SULID-V2: 01J75Y6K0A16WE3XQN92QPP701
SULID-V2: 01J75Y6K0AN12FBKX56YGZF301
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

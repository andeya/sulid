#![allow(dead_code)]
#![allow(unused_imports)]
//! # sulid
//!
//! This is a Rust implementation of the SULID (Snowflake-inspired Universally Unique Lexicographically Sortable Identifier).
//!
//! ## Quickstart
//!
//! ```rust
//! # use sulid::Sulid;
//! // Generate a sulid
//! # let sulid = Sulid::default();
//!
//! // Generate a string for a sulid
//! let s = sulid.to_string();
//!
//! // Create from a String
//! let res = Sulid::from_string(&s);
//! assert_eq!(sulid, res.unwrap());
//!
//! // Or using FromStr
//! let res = s.parse();
//! assert_eq!(sulid, res.unwrap());
//! ```

use crate::{DecodeError, ULID_LEN};
use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;
use ulid::Ulid;

/// Create a right-aligned bitmask of $len bits
macro_rules! bitmask {
    ($len:expr => $int_ty:ty) => {
        ((1u128 << $len) - 1) as $int_ty
    };
}
// Allow other modules to use the macro
pub(crate) use bitmask;

/// A Sulid is a unique 128-bit lexicographically sortable identifier
///
/// Canonically, it is represented as a 26 character Crockford Base32 encoded
/// string.
///
/// Of the 128-bits, the first 48 are a unix timestamp in milliseconds. The
/// next 70 bits are random. The remaining 10 bits are divided into
/// 5-bit data center ID and 5-bit machine ID.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Sulid {
    /// Millisecond-Sulid
    MS(Ulid),
    /// Microsecond-Sulid
    US(Ulid),
}

impl Sulid {
    /// The number of bits in a Sulid's time portion.
    /// Millisecond timestamp, which can represent up to the year 2109 AD.
    pub const TIME_BITS_MS: u8 = 42;
    /// The number of bits in a Sulid's random portion is designed for the millisecond timestamp plan.
    pub const RAND_BITS_MS: u8 = 76;
    /// The number of bits in a Sulid's time portion.
    /// Microsecond timestamp, which can represent up to the year 2112 AD.
    pub const TIME_BITS_US: u8 = 52;
    /// The number of bits in a Sulid's random portion is designed for the microsecond timestamp plan.
    pub const RAND_BITS_US: u8 = 66;
    /// The number of bits for data center ID
    pub const DATA_CENTER_BITS: u8 = 5;
    /// The number of bits for machine ID
    pub const MACHINE_BITS: u8 = 5;
    /// The number of bits for worker ID, which is a combination of data_center_id and machine_id.
    pub const WORKER_BITS: u8 = 10;

    /// Create a Sulid from integer representation.
    #[inline]
    pub const fn from_u128(u: u128, ts_type: TimestampType) -> Self {
        match ts_type {
            TimestampType::MS => Self::MS(Ulid(u)),
            TimestampType::US => Self::US(Ulid(u)),
        }
    }

    /// Gets the integer representation
    pub const fn as_u128(&self) -> u128 {
        match self {
            Sulid::MS(ulid) => ulid.0,
            Sulid::US(ulid) => ulid.0,
        }
    }

    /// Create a Sulid from separated parts (using millisecond).
    ///
    /// NOTE: Any overflow bits in the given args are discarded
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let sulid = Sulid::from_string("01D39ZY06FGSCTVN4T2V9PKHFZ").unwrap();
    ///
    /// let sulid2 = Sulid::from_parts(sulid.timestamp(), sulid.random(), sulid.worker_id());
    ///
    /// assert_eq!(sulid, sulid2);
    /// ```
    #[inline]
    pub fn from_parts(timestamp: Timestamp, random: u128, worker_id: WorkId) -> Sulid {
        let worker_id = match worker_id {
            WorkId::One(id) => id,
            WorkId::Two {
                data_center_id,
                machine_id,
            } => {
                let bitmask_data_center_id: u8 = bitmask!(Self::DATA_CENTER_BITS => u8);
                let bitmask_machine_id: u8 = bitmask!(Self::MACHINE_BITS => u8);
                #[cfg(feature = "assert")]
                {
                    assert!(
                        data_center_id <= bitmask_data_center_id,
                        "data_center_id must be in the range 0-{bitmask_data_center_id}"
                    );
                    assert!(
                        machine_id <= bitmask_machine_id,
                        "machine_id must be in the range 0-{bitmask_machine_id}"
                    );
                }
                ((data_center_id as u16) << Self::MACHINE_BITS) | (machine_id as u16)
            }
        };
        let (ts, time_bits, rand_bit) = match timestamp {
            Timestamp::MS(ts) => (ts, Self::TIME_BITS_MS, Self::RAND_BITS_MS),
            Timestamp::US(ts) => (ts, Self::TIME_BITS_US, Self::RAND_BITS_US),
        };
        let bitmask_timestamp: u64 = bitmask!(time_bits => u64);
        let bitmask_random: u128 = bitmask!(rand_bit => u128);
        let bitmask_worker_id: u16 = bitmask!(Self::WORKER_BITS => u16);
        #[cfg(feature = "assert")]
        {
            assert!(
                ts <= bitmask_timestamp,
                "timestamp must be in the range 0-{bitmask_timestamp}"
            );
            assert!(
                random <= bitmask_random,
                "random must be in the range 0-{bitmask_random}"
            );
            assert!(
                worker_id <= bitmask_worker_id,
                "worker_id must be in the range 0-{bitmask_worker_id}"
            );
        }

        let time_part = (ts & bitmask_timestamp) as u128;
        let rand_part = random & bitmask_random;
        let worker_part = (worker_id & bitmask_worker_id) as u128;

        let id = Ulid(
            (time_part << (Self::RAND_BITS_MS + Self::DATA_CENTER_BITS + Self::MACHINE_BITS))
                | (rand_part << (Self::DATA_CENTER_BITS + Self::MACHINE_BITS))
                | worker_part,
        );

        match timestamp {
            Timestamp::MS(_) => Sulid::MS(id),
            Timestamp::US(_) => Sulid::US(id),
        }
    }

    /// Creates a Sulid from a Crockford Base32 encoded string
    ///
    /// An DecodeError will be returned when the given string is not formatted
    /// properly.
    ///
    /// # Example
    /// ```rust
    /// use sulid::{Sulid,TimestampType};
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let result = Sulid::from_string(text, TimestampType::MS);
    ///
    /// assert!(result.is_ok());
    /// assert_eq!(&result.unwrap().to_string(), text);
    /// ```
    #[inline]
    pub const fn from_string(encoded: &str, ts_type: TimestampType) -> Result<Sulid, DecodeError> {
        match Ulid::from_string(encoded) {
            Ok(int_val) => Ok(match ts_type {
                TimestampType::MS => Sulid::MS(int_val),
                TimestampType::US => Sulid::US(int_val),
            }),
            Err(err) => Err(err),
        }
    }

    /// The 'nil Sulid'.
    ///
    /// The nil Sulid is special form of Sulid that is specified to have
    /// all 128 bits set to zero.
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let sulid = Sulid::nil(TimestampType::MS);
    ///
    /// assert_eq!(
    ///     sulid.to_string(),
    ///     "00000000000000000000000000"
    /// );
    /// ```
    #[inline]
    pub const fn nil(ts_type: TimestampType) -> Sulid {
        match ts_type {
            TimestampType::MS => Sulid::MS(Ulid::nil()),
            TimestampType::US => Sulid::US(Ulid::nil()),
        }
    }

    const fn inner(&self) -> &Ulid {
        match self {
            Sulid::MS(ulid) => ulid,
            Sulid::US(ulid) => ulid,
        }
    }

    /// Gets the microsecond timestamp section of this sulid.
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use std::time::{SystemTime, Duration};
    /// use sulid::Sulid;
    ///
    /// let dt = SystemTime::now();
    /// let ts = Timestamp::from_u128(dt.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis());
    /// let sulid = Sulid::from_parts(ts, 1, WorkId::One(11));
    /// assert_eq!(u128::from(sulid.timestamp().as_u64()), dt.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis());
    /// # }
    /// ```
    pub const fn timestamp(&self) -> Timestamp {
        match self {
            Sulid::MS(ulid) => {
                Timestamp::MS((ulid.0 >> (Self::RAND_BITS_MS + Self::WORKER_BITS)) as u64)
            }
            Sulid::US(ulid) => {
                Timestamp::US((ulid.0 >> (Self::RAND_BITS_US + Self::WORKER_BITS)) as u64)
            }
        }
    }

    /// Gets the random section of this sulid
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let sulid = Sulid::from_string(text).unwrap();
    /// let sulid_next = sulid.increment().unwrap();
    ///
    /// assert_eq!(sulid.random() + 1, sulid_next.random());
    /// ```
    pub const fn random(&self) -> u128 {
        (self.inner().0 >> (Self::WORKER_BITS)) & bitmask!(Self::RAND_BITS_MS => u128)
    }

    /// Gets the data center ID portion of this sulid
    pub const fn data_center_id(&self) -> u8 {
        ((self.inner().0 >> Self::MACHINE_BITS) & bitmask!(Self::DATA_CENTER_BITS => u128)) as u8
    }

    /// Gets the machine ID portion of this sulid
    pub const fn machine_id(&self) -> u8 {
        (self.inner().0 & bitmask!(Self::MACHINE_BITS => u128)) as u8
    }

    /// Gets the worker ID portion of this sulid
    pub const fn worker_id(&self) -> u16 {
        (self.inner().0 & bitmask!(Self::WORKER_BITS => u128)) as u16
    }

    /// Creates a Crockford Base32 encoded string that represents this Sulid
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let sulid = Sulid::from_string(text).unwrap();
    ///
    /// let mut buf = [0; sulid::ULID_LEN];
    /// let new_text = sulid.array_to_str(&mut buf);
    ///
    /// assert_eq!(new_text, text);
    /// ```
    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULID_LEN]) -> &'buf mut str {
        self.inner().array_to_str(buf)
    }

    /// Test if the Sulid is nil
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let sulid = Sulid::from_u128(1);
    /// assert!(!sulid.is_nil());
    ///
    /// let nil = Sulid::nil();
    /// assert!(nil.is_nil());
    /// ```
    #[inline]
    pub const fn is_nil(&self) -> bool {
        self.inner().is_nil()
    }

    /// Increment the random number, make sure that the ts stays the same.
    pub const fn increment(&self) -> Option<Sulid> {
        match self {
            Sulid::MS(ulid) => {
                const MAX_RANDOM: u128 = bitmask!(Sulid::RAND_BITS_MS => u128);
                if ((ulid.0 >> Sulid::WORKER_BITS) & MAX_RANDOM) == MAX_RANDOM {
                    None
                } else {
                    Some(Sulid::MS(Ulid(ulid.0 + (1 << Sulid::WORKER_BITS))))
                }
            }
            Sulid::US(ulid) => {
                const MAX_RANDOM: u128 = bitmask!(Sulid::RAND_BITS_US => u128);
                if ((ulid.0 >> Sulid::WORKER_BITS) & MAX_RANDOM) == MAX_RANDOM {
                    None
                } else {
                    Some(Sulid::US(Ulid(ulid.0 + (1 << Sulid::WORKER_BITS))))
                }
            }
        }
    }

    /// Creates a Sulid using the provided bytes array.
    ///
    /// # Example
    /// ```
    /// use sulid::Sulid;
    /// let bytes = [0xFF; 16];
    ///
    /// let sulid = Sulid::from_bytes(bytes, Timestamp::MS);
    ///
    /// assert_eq!(
    ///     sulid.to_string(),
    ///     "7ZZZZZZZZZZZZZZZZZZZZZZZZZ"
    /// );
    /// ```
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16], ts_type: TimestampType) -> Sulid {
        match ts_type {
            TimestampType::MS => Self::MS(Ulid::from_bytes(bytes)),
            TimestampType::US => Self::US(Ulid::from_bytes(bytes)),
        }
    }

    /// Returns the bytes of the Sulid in big-endian order.
    ///
    /// # Example
    /// ```
    /// use sulid::Sulid;
    ///
    /// let text = "7ZZZZZZZZZZZZZZZZZZZZZZZZZ";
    /// let sulid = Sulid::from_string(text).unwrap();
    ///
    /// assert_eq!(sulid.to_bytes(), [0xFF; 16]);
    /// ```
    #[inline]
    pub const fn to_bytes(&self) -> [u8; 16] {
        self.inner().to_bytes()
    }
}

/// Timestamp type, millisecond or microsecond.
#[derive(Debug, Clone, Copy)]
pub enum TimestampType {
    /// Millisecond timestamp
    MS,
    /// Microsecond timestamp
    US,
}
impl TimestampType {
    /// New Timestamp from u64 type.
    pub const fn new_ts_u64(self, ts: u64) -> Timestamp {
        Timestamp::from_u64(ts, self)
    }
    /// New Timestamp from u128 type.
    pub const fn new_ts_u128(self, ts: u128) -> Timestamp {
        Timestamp::from_u128(ts, self)
    }
}

/// Timestamp, whose type may be millisecond or microsecond.
#[derive(Debug, Clone, Copy)]
pub enum Timestamp {
    /// Millisecond timestamp
    MS(u64),
    /// Microsecond timestamp
    US(u64),
}

impl Timestamp {
    /// New Timestamp from u64 type.
    #[inline]
    pub const fn from_u64(ts: u64, ts_type: TimestampType) -> Self {
        match ts_type {
            TimestampType::MS => Self::MS(ts),
            TimestampType::US => Self::US(ts),
        }
    }
    /// New Timestamp from u128 type.
    #[inline]
    pub const fn from_u128(ts: u128, ts_type: TimestampType) -> Self {
        Self::from_u64(ts as u64, ts_type)
    }
    /// Convert to u64.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        match self {
            Timestamp::MS(v) => v,
            Timestamp::US(v) => v,
        }
    }
    /// Convert to u128.
    #[inline]
    pub const fn as_u128(self) -> u128 {
        self.as_u64() as u128
    }
}

/// A 10-bit work ID, which can be composed of a 5-bit data center ID and a 5-bit machine ID.
#[derive(Debug, Clone, Copy)]
pub enum WorkId {
    /// 10 bits, the combination of data_center_id and machine_id.
    One(u16),
    /// The work ID composed of the data center ID and the machine ID.
    Two {
        /// 5 bits, identifying the data center.
        data_center_id: u8,
        /// 5 bits, identifying the machine within the data center.
        machine_id: u8,
    },
}
impl WorkId {
    /// New WorkId from work_id.
    pub const fn new_one(work_id: u16) -> Self {
        Self::One(work_id)
    }
    /// New WorkId from data_center_id and machine_id.
    pub const fn new_two(data_center_id: u8, machine_id: u8) -> Self {
        Self::Two {
            data_center_id,
            machine_id,
        }
    }
}
impl From<u16> for WorkId {
    fn from(value: u16) -> Self {
        Self::One(value)
    }
}

impl From<(u8, u8)> for WorkId {
    fn from((data_center_id, machine_id): (u8, u8)) -> Self {
        Self::Two {
            data_center_id,
            machine_id,
        }
    }
}

impl From<(Timestamp, u128, u8, u8)> for Sulid {
    /// NOTE: It is only meaningful for v1.
    fn from((timestamp, random, data_center_id, machine_id): (Timestamp, u128, u8, u8)) -> Self {
        Sulid::from_parts(
            timestamp,
            random,
            WorkId::Two {
                data_center_id,
                machine_id,
            },
        )
    }
}

impl From<Sulid> for (Timestamp, u128, u8, u8) {
    /// NOTE: It is only meaningful for v1.
    fn from(sulid: Sulid) -> (Timestamp, u128, u8, u8) {
        (
            sulid.timestamp(),
            sulid.random(),
            sulid.data_center_id(),
            sulid.machine_id(),
        )
    }
}

impl From<(Timestamp, u128, u16)> for Sulid {
    /// NOTE: It is only meaningful for v2.
    fn from((timestamp, random, worker_id): (Timestamp, u128, u16)) -> Self {
        Sulid::from_parts(timestamp, random, WorkId::One(worker_id))
    }
}

impl From<Sulid> for (Timestamp, u128, u16) {
    /// NOTE: It is only meaningful for v2.
    fn from(sulid: Sulid) -> (Timestamp, u128, u16) {
        (sulid.timestamp(), sulid.random(), sulid.worker_id())
    }
}

impl From<Sulid> for u128 {
    fn from(sulid: Sulid) -> u128 {
        sulid.inner().0
    }
}

impl From<Sulid> for [u8; 16] {
    fn from(sulid: Sulid) -> Self {
        sulid.inner().to_bytes()
    }
}

impl fmt::Display for Sulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut buffer = [0; ULID_LEN];
        write!(f, "{}", self.array_to_str(&mut buffer))
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_static() {
        let mut s = [0u8; ULID_LEN];
        let s = Sulid::from_u128(0x41414141414141414141414141414141, TimestampType::MS)
            .array_to_str(&mut s);
        let u = Sulid::from_string(&s, TimestampType::MS).unwrap();
        assert_eq!(s, "21850M2GA1850M2GA1850M2GA1");
        assert_eq!(u.as_u128(), 0x41414141414141414141414141414141);
    }

    #[test]
    fn test_increment() {
        let mut s = [0u8; ULID_LEN];

        let sulid = Sulid::from_string("01BX5ZZKBKAZZZZZZZZZZZZZZZ", TimestampType::MS).unwrap();
        let sulid = sulid.increment().unwrap();
        assert_eq!("01BX5ZZKBKB0000000000000ZZ", sulid.array_to_str(&mut s));

        let sulid = Sulid::from_string("01BX5ZZKBKZZZZZZZZZZZZZXZX", TimestampType::MS).unwrap();
        let sulid = sulid.increment().unwrap();
        s.fill(0);
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZYZX", sulid.array_to_str(&mut s));

        let sulid = sulid.increment().unwrap();
        s.fill(0);
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZZZX", sulid.array_to_str(&mut s));
    }

    #[test]
    fn test_increment_overflow() {
        let sulid = Sulid::from_u128(u128::max_value(), TimestampType::MS);
        assert_eq!("7ZZZZZZZZZZZZZZZZZZZZZZZZZ", sulid.to_string());
        assert!(sulid.increment().is_none());
    }

    #[test]
    fn can_into_thing() {
        let sulid = Sulid::from_string("01FKMG6GAG0PJANMWFN84TNXCD", TimestampType::MS).unwrap();
        let u: u128 = sulid.into();
        let uu: (Timestamp, u128, u8, u8) = sulid.into();
        let uu2: (Timestamp, u128, u16) = sulid.into();
        let bytes: [u8; 16] = sulid.into();
        assert_eq!(Sulid::from_u128(u, TimestampType::MS), sulid);
        assert_eq!(Sulid::from(uu), sulid);
        assert_eq!(Sulid::from(uu2), sulid);
        assert_eq!(Sulid::from_bytes(bytes, TimestampType::MS), sulid);

        #[cfg(feature = "std")]
        {
            let s: String = sulid.into();
            assert_eq!(Sulid::from_string(&s, TimestampType::MS).unwrap(), sulid);
        }
    }
}

#[cfg(feature = "std")]
pub(crate) mod std_feature {
    use crate::{sulid::bitmask, Sulid, Timestamp, TimestampType, WorkId};
    use std::time::{Duration, SystemTime};

    impl Timestamp {
        /// Return the system time.
        pub fn system_time(self) -> SystemTime {
            SystemTime::UNIX_EPOCH
                + match self {
                    Timestamp::MS(ts) => Duration::from_millis(ts),
                    Timestamp::US(ts) => Duration::from_micros(ts),
                }
        }
    }

    impl From<Sulid> for String {
        fn from(sulid: Sulid) -> String {
            sulid.to_string()
        }
    }

    impl Sulid {
        /// Creates a new Sulid with the current time (UTC)
        ///
        /// Using this function to generate Sulids will not guarantee monotonic sort order.
        /// See [sulid::SulidGenerator] for a monotonic sort order.
        /// # Example
        /// ```rust
        /// use sulid::Sulid;
        ///
        /// let my_sulid = Sulid::new(0.into(), TimestampType::MS);
        /// ```
        pub fn new(worker_id: WorkId, ts_type: TimestampType) -> Sulid {
            Sulid::from_datetime(now(), worker_id, ts_type)
        }

        /// Creates a new Sulid using data from the given random number generator
        ///
        /// # Example
        /// ```rust
        /// use rand::prelude::*;
        /// use sulid::Sulid;
        ///
        /// let mut rng = StdRng::from_entropy();
        /// let sulid = Sulid::with_source(&mut rng, 0, 0);
        /// ```
        pub fn with_source<R: rand::Rng>(
            source: &mut R,
            worker_id: WorkId,
            ts_type: TimestampType,
        ) -> Sulid {
            Sulid::from_datetime_source(now(), source, worker_id, ts_type)
        }

        /// Creates a new Sulid with the given datetime
        ///
        /// This can be useful when migrating data to use Sulid identifiers.
        ///
        /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
        /// as earlier times are not valid for a Sulid timestamp
        ///
        /// # Example
        /// ```rust
        /// use std::time::{SystemTime, Duration};
        /// use sulid::Sulid;
        ///
        /// let sulid = Sulid::from_datetime(SystemTime::now(), 0);
        /// ```
        pub fn from_datetime(
            datetime: SystemTime,
            worker_id: WorkId,
            ts_type: TimestampType,
        ) -> Sulid {
            Sulid::from_datetime_source(datetime, &mut rand::thread_rng(), worker_id, ts_type)
        }

        /// Creates a new Sulid with the given datetime and random number generator
        ///
        /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
        /// as earlier times are not valid for a Sulid timestamp
        ///
        /// # Example
        /// ```rust
        /// use std::time::{SystemTime, Duration};
        /// use rand::prelude::*;
        /// use sulid::Sulid;
        ///
        /// let mut rng = StdRng::from_entropy();
        /// let sulid = Sulid::from_datetime_source(SystemTime::now(), &mut rng, 0, WorkId::One(0));
        /// ```
        pub fn from_datetime_source<R>(
            datetime: SystemTime,
            source: &mut R,
            worker_id: WorkId,
            ts_type: TimestampType,
        ) -> Sulid
        where
            R: rand::Rng + ?Sized,
        {
            let timestamp = datetime
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis();
            let timebits = (timestamp & bitmask!(Self::TIME_BITS_MS => u128)) as u64;
            let randbits = (source.gen::<u128>() & bitmask!(Self::RAND_BITS_MS => u128)) as u128;
            Sulid::from_parts(ts_type.new_ts_u64(timebits), randbits, worker_id)
        }

        /// Gets the datetime of when this Sulid was created accurate to 1ms
        ///
        /// # Example
        /// ```rust
        /// use std::time::{SystemTime, Duration};
        /// use sulid::Sulid;
        ///
        /// let dt = SystemTime::now();
        /// let sulid = Sulid::from_datetime(dt, 0, 0);
        ///
        /// assert!(
        ///     dt + Duration::from_millis(1) >= sulid.datetime()
        ///     && dt - Duration::from_millis(1) <= sulid.datetime()
        /// );
        /// ```
        pub fn datetime(&self) -> SystemTime {
            self.timestamp().system_time()
        }
        /// Creates a Crockford Base32 encoded string that represents this Sulid
        ///
        /// # Example
        /// ```rust
        /// use sulid::Sulid;
        ///
        /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
        /// let sulid = Sulid::from_string(text).unwrap();
        ///
        /// assert_eq!(&sulid.to_string(), text);
        /// ```
        #[allow(clippy::inherent_to_string_shadow_display)] // Significantly faster than Display::to_string
        pub fn to_string(&self) -> String {
            self.inner().to_string()
        }
    }

    fn now() -> std::time::SystemTime {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            use web_time::web::SystemTimeExt;
            return web_time::SystemTime::now().to_std();
        }
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        return std::time::SystemTime::now();
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{DecodeError, EncodeError};

        #[test]
        fn can_display_things() {
            println!("{}", Sulid::nil(TimestampType::MS));
            println!("{}", EncodeError::BufferTooSmall);
            println!("{}", DecodeError::InvalidLength);
            println!("{}", DecodeError::InvalidChar);
        }

        #[test]
        fn test_dynamic() {
            let sulid = Sulid::new(0.into(), TimestampType::MS);
            let encoded = sulid.to_string();
            let sulid2 =
                Sulid::from_string(&encoded, TimestampType::MS).expect("failed to deserialize");

            println!("{}", encoded);
            println!("{:?}", sulid);
            println!("{:?}", sulid2);
            assert_eq!(sulid, sulid2);
        }

        #[test]
        fn test_source() {
            use rand::rngs::mock::StepRng;
            let mut source = StepRng::new(123, 0);

            let u1 = Sulid::with_source(&mut source, 0.into(), TimestampType::MS);
            let dt = SystemTime::now() + Duration::from_millis(1);
            let u2 = Sulid::from_datetime_source(dt, &mut source, 0.into(), TimestampType::MS);
            let u3 = Sulid::from_datetime_source(dt, &mut source, 0.into(), TimestampType::MS);

            assert!(u1 < u2);
            assert_eq!(u2, u3);
        }

        #[test]
        fn test_order() {
            let dt = SystemTime::now();
            let sulid1 = Sulid::from_datetime(dt, 0.into(), TimestampType::MS);
            let sulid2 =
                Sulid::from_datetime(dt + Duration::from_millis(1), 0.into(), TimestampType::MS);
            assert!(sulid1 < sulid2);
        }

        #[test]
        fn test_datetime() {
            let dt = SystemTime::now();
            let sulid = Sulid::from_datetime(dt, 0.into(), TimestampType::MS);

            println!("{:?}, {:?}", dt, sulid.datetime());
            assert!(sulid.datetime() <= dt);
            assert!(sulid.datetime() + Duration::from_millis(1) >= dt);
        }

        #[test]
        fn test_timestamp() {
            let dt = SystemTime::now();
            let sulid = Sulid::from_datetime(dt, 0.into(), TimestampType::MS);
            let ts = dt
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            assert_eq!(sulid.timestamp().as_u128(), ts);
        }

        #[test]
        fn nil_is_at_unix_epoch() {
            assert_eq!(
                Sulid::nil(TimestampType::MS).datetime(),
                SystemTime::UNIX_EPOCH
            );
        }

        #[test]
        fn truncates_at_unix_epoch() {
            if let Some(before_epoch) = SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(100))
            {
                assert!(before_epoch < SystemTime::UNIX_EPOCH);
                assert_eq!(
                    Sulid::from_datetime(before_epoch, 0.into(), TimestampType::MS).datetime(),
                    SystemTime::UNIX_EPOCH
                );
            } else {
                // Prior dates are not representable (e.g. wasm32-wasi)
            }
        }
    }
}

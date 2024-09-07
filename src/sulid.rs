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
pub struct Sulid(Ulid);

impl Sulid {
    /// The number of bits in a Sulid's time portion
    pub const TIME_BITS: u8 = 48;
    /// The number of bits in a Sulid's random portion
    pub const RAND_BITS: u8 = 70;
    /// The number of bits for data center ID
    pub const DATA_CENTER_BITS: u8 = 5;
    /// The number of bits for machine ID
    pub const MACHINE_BITS: u8 = 5;

    /// Create a Sulid from integer representation.
    pub fn from_u128(u: u128) -> Self {
        Self(Ulid(u))
    }

    /// Gets the integer representation
    pub fn u128(&self) -> u128 {
        self.0 .0
    }

    /// Create a Sulid from separated parts.
    ///
    /// NOTE: Any overflow bits in the given args are discarded
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let sulid = Sulid::from_string("01D39ZY06FGSCTVN4T2V9PKHFZ").unwrap();
    ///
    /// let sulid2 = Sulid::from_parts(sulid.timestamp_ms(), sulid.random(), sulid.data_center_id(), sulid.machine_id());
    ///
    /// assert_eq!(sulid, sulid2);
    /// ```
    #[inline]
    pub const fn from_parts(
        timestamp_ms: u64,
        random: u128,
        data_center_id: u8,
        machine_id: u8,
    ) -> Sulid {
        let bitmask_timestamp_ms: u64 = bitmask!(Self::TIME_BITS => u64);
        let bitmask_random: u128 = bitmask!(Self::RAND_BITS => u128);
        let bitmask_data_center_id: u8 = bitmask!(Self::DATA_CENTER_BITS => u8);
        let bitmask_machine_id: u8 = bitmask!(Self::MACHINE_BITS => u8);

        #[cfg(feature = "assert")]
        {
            assert!(
                timestamp_ms <= bitmask_timestamp_ms,
                "timestamp_ms must be in the range 0-281474976710655"
            );
            assert!(
                random <= bitmask_random,
                "random must be in the range 0-1180591620717411303423"
            );
            assert!(
                data_center_id <= bitmask_data_center_id,
                "data_center_id must be in the range 0-31"
            );
            assert!(
                machine_id <= bitmask_machine_id,
                "machine_id must be in the range 0-31"
            );
        }

        let time_part = (timestamp_ms & bitmask_timestamp_ms) as u128;
        let rand_part = random & bitmask_random;
        let data_center_part = (data_center_id & bitmask_data_center_id) as u128;
        let machine_part = (machine_id & bitmask_machine_id) as u128;

        Sulid(Ulid(
            (time_part << (Self::RAND_BITS + Self::DATA_CENTER_BITS + Self::MACHINE_BITS))
                | (rand_part << (Self::DATA_CENTER_BITS + Self::MACHINE_BITS))
                | (data_center_part << Self::MACHINE_BITS)
                | machine_part,
        ))
    }

    /// Creates a Sulid from a Crockford Base32 encoded string
    ///
    /// An DecodeError will be returned when the given string is not formatted
    /// properly.
    ///
    /// # Example
    /// ```rust
    /// use sulid::Sulid;
    ///
    /// let text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
    /// let result = Sulid::from_string(text);
    ///
    /// assert!(result.is_ok());
    /// assert_eq!(&result.unwrap().to_string(), text);
    /// ```
    #[inline]
    pub const fn from_string(encoded: &str) -> Result<Sulid, DecodeError> {
        match Ulid::from_string(encoded) {
            Ok(int_val) => Ok(Sulid(int_val)),
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
    /// let sulid = Sulid::nil();
    ///
    /// assert_eq!(
    ///     sulid.to_string(),
    ///     "00000000000000000000000000"
    /// );
    /// ```
    #[inline]
    pub const fn nil() -> Sulid {
        Sulid(Ulid::nil())
    }

    /// Gets the timestamp section of this sulid
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use std::time::{SystemTime, Duration};
    /// use sulid::Sulid;
    ///
    /// let dt = SystemTime::now();
    /// let sulid = Sulid::from_parts(dt.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis() as u64, 1, 1, 1);
    ///
    /// assert_eq!(u128::from(sulid.timestamp_ms()), dt.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis());
    /// # }
    /// ```
    pub const fn timestamp_ms(&self) -> u64 {
        (self.0 .0 >> (Self::RAND_BITS + Self::DATA_CENTER_BITS + Self::MACHINE_BITS)) as u64
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
        (self.0 .0 >> (Self::DATA_CENTER_BITS + Self::MACHINE_BITS))
            & bitmask!(Self::RAND_BITS => u128)
    }

    /// Gets the data center ID portion of this sulid
    pub const fn data_center_id(&self) -> u8 {
        ((self.0 .0 >> Self::MACHINE_BITS) & bitmask!(Self::DATA_CENTER_BITS => u128)) as u8
    }

    /// Gets the machine ID portion of this sulid
    pub const fn machine_id(&self) -> u8 {
        (self.0 .0 & bitmask!(Self::MACHINE_BITS => u128)) as u8
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
        self.0.array_to_str(buf)
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
        self.0.is_nil()
    }

    /// Increment the random number, make sure that the ts millis stays the same
    pub const fn increment(&self) -> Option<Sulid> {
        const MAX_RANDOM: u128 = bitmask!(Sulid::RAND_BITS => u128);

        if ((self.0 .0 >> (Sulid::DATA_CENTER_BITS + Sulid::MACHINE_BITS)) & MAX_RANDOM)
            == MAX_RANDOM
        {
            None
        } else {
            Some(Sulid(Ulid(
                self.0 .0 + (1 << (Sulid::DATA_CENTER_BITS + Sulid::MACHINE_BITS)),
            )))
        }
    }

    /// Creates a Sulid using the provided bytes array.
    ///
    /// # Example
    /// ```
    /// use sulid::Sulid;
    /// let bytes = [0xFF; 16];
    ///
    /// let sulid = Sulid::from_bytes(bytes);
    ///
    /// assert_eq!(
    ///     sulid.to_string(),
    ///     "7ZZZZZZZZZZZZZZZZZZZZZZZZZ"
    /// );
    /// ```
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Sulid {
        Self(Ulid::from_bytes(bytes))
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
        self.0.to_bytes()
    }
}

impl Default for Sulid {
    fn default() -> Self {
        Sulid::nil()
    }
}

impl From<(u64, u128, u8, u8)> for Sulid {
    fn from((timestamp_ms, random, data_center_id, machine_id): (u64, u128, u8, u8)) -> Self {
        Sulid::from_parts(timestamp_ms, random, data_center_id, machine_id)
    }
}

impl From<Sulid> for (u64, u128, u8, u8) {
    fn from(sulid: Sulid) -> (u64, u128, u8, u8) {
        (
            sulid.timestamp_ms(),
            sulid.random(),
            sulid.data_center_id(),
            sulid.machine_id(),
        )
    }
}

impl From<u128> for Sulid {
    fn from(value: u128) -> Sulid {
        Sulid(Ulid(value))
    }
}

impl From<Sulid> for u128 {
    fn from(sulid: Sulid) -> u128 {
        sulid.0 .0
    }
}

impl From<[u8; 16]> for Sulid {
    fn from(bytes: [u8; 16]) -> Self {
        Self(Ulid::from_bytes(bytes))
    }
}

impl From<Sulid> for [u8; 16] {
    fn from(sulid: Sulid) -> Self {
        sulid.0.to_bytes()
    }
}

impl FromStr for Sulid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Sulid::from_string(s)
    }
}

impl TryFrom<&'_ str> for Sulid {
    type Error = DecodeError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        Sulid::from_string(value)
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
        let s = Sulid::from_u128(0x41414141414141414141414141414141).array_to_str(&mut s);
        let u = Sulid::from_string(&s).unwrap();
        assert_eq!(s, "21850M2GA1850M2GA1850M2GA1");
        assert_eq!(u.u128(), 0x41414141414141414141414141414141);
    }

    #[test]
    fn test_increment() {
        let mut s = [0u8; ULID_LEN];

        let sulid = Sulid::from_string("01BX5ZZKBKAZZZZZZZZZZZZZZZ").unwrap();
        let sulid = sulid.increment().unwrap();
        assert_eq!("01BX5ZZKBKB0000000000000ZZ", sulid.array_to_str(&mut s));

        let sulid = Sulid::from_string("01BX5ZZKBKZZZZZZZZZZZZZXZX").unwrap();
        let sulid = sulid.increment().unwrap();
        s.fill(0);
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZYZX", sulid.array_to_str(&mut s));

        let sulid = sulid.increment().unwrap();
        s.fill(0);
        assert_eq!("01BX5ZZKBKZZZZZZZZZZZZZZZX", sulid.array_to_str(&mut s));
        assert!(sulid.increment().is_none());
    }

    #[test]
    fn test_increment_overflow() {
        let sulid = Sulid::from_u128(u128::max_value());
        assert!(sulid.increment().is_none());
    }

    #[test]
    fn can_into_thing() {
        let sulid = Sulid::from_str("01FKMG6GAG0PJANMWFN84TNXCD").unwrap();
        let u: u128 = sulid.into();
        let uu: (u64, u128, u8, u8) = sulid.into();
        let bytes: [u8; 16] = sulid.into();
        assert_eq!(Sulid::from(u), sulid);
        assert_eq!(Sulid::from(uu), sulid);
        assert_eq!(Sulid::from(bytes), sulid);

        #[cfg(feature = "std")]
        {
            let s: String = sulid.into();
            assert_eq!(Sulid::from_str(&s).unwrap(), sulid);
        }
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Sulid::default(), Sulid::nil());
    }
}

#[cfg(feature = "std")]
pub(crate) mod std_feature {
    use crate::{sulid::bitmask, Sulid};
    use std::time::{Duration, SystemTime};

    impl From<Sulid> for String {
        fn from(sulid: Sulid) -> String {
            sulid.to_string()
        }
    }

    impl Sulid {
        /// Creates a new Sulid with the current time (UTC)
        ///
        /// Using this function to generate Sulids will not guarantee monotonic sort order.
        /// See [sulid::Generator] for a monotonic sort order.
        /// # Example
        /// ```rust
        /// use sulid::Sulid;
        ///
        /// let my_sulid = Sulid::new(0, 0);
        /// ```
        pub fn new(data_center_id: u8, machine_id: u8) -> Sulid {
            Sulid::from_datetime(now(), data_center_id, machine_id)
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
            data_center_id: u8,
            machine_id: u8,
        ) -> Sulid {
            Sulid::from_datetime_with_source(now(), source, data_center_id, machine_id)
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
        /// let sulid = Sulid::from_datetime(SystemTime::now(), 0, 0);
        /// ```
        pub fn from_datetime(datetime: SystemTime, data_center_id: u8, machine_id: u8) -> Sulid {
            Sulid::from_datetime_with_source(
                datetime,
                &mut rand::thread_rng(),
                data_center_id,
                machine_id,
            )
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
        /// let sulid = Sulid::from_datetime_with_source(SystemTime::now(), &mut rng, 0, 0);
        /// ```
        pub fn from_datetime_with_source<R>(
            datetime: SystemTime,
            source: &mut R,
            data_center_id: u8,
            machine_id: u8,
        ) -> Sulid
        where
            R: rand::Rng + ?Sized,
        {
            let timestamp = datetime
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis();
            let timebits = (timestamp & bitmask!(Self::TIME_BITS => u128)) as u64;
            let randbits = (source.gen::<u128>() & bitmask!(Self::RAND_BITS => u128)) as u128;
            Sulid::from_parts(timebits, randbits, data_center_id, machine_id)
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
            let stamp = self.timestamp_ms();
            SystemTime::UNIX_EPOCH + Duration::from_millis(stamp)
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
            self.0.to_string()
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
            println!("{}", Sulid::nil());
            println!("{}", EncodeError::BufferTooSmall);
            println!("{}", DecodeError::InvalidLength);
            println!("{}", DecodeError::InvalidChar);
        }

        #[test]
        fn test_dynamic() {
            let sulid = Sulid::new(0, 0);
            let encoded = sulid.to_string();
            let sulid2 = Sulid::from_string(&encoded).expect("failed to deserialize");

            println!("{}", encoded);
            println!("{:?}", sulid);
            println!("{:?}", sulid2);
            assert_eq!(sulid, sulid2);
        }

        #[test]
        fn test_source() {
            use rand::rngs::mock::StepRng;
            let mut source = StepRng::new(123, 0);

            let u1 = Sulid::with_source(&mut source, 0, 0);
            let dt = SystemTime::now() + Duration::from_millis(1);
            let u2 = Sulid::from_datetime_with_source(dt, &mut source, 0, 0);
            let u3 = Sulid::from_datetime_with_source(dt, &mut source, 0, 0);

            assert!(u1 < u2);
            assert_eq!(u2, u3);
        }

        #[test]
        fn test_order() {
            let dt = SystemTime::now();
            let sulid1 = Sulid::from_datetime(dt, 0, 0);
            let sulid2 = Sulid::from_datetime(dt + Duration::from_millis(1), 0, 0);
            assert!(sulid1 < sulid2);
        }

        #[test]
        fn test_datetime() {
            let dt = SystemTime::now();
            let sulid = Sulid::from_datetime(dt, 0, 0);

            println!("{:?}, {:?}", dt, sulid.datetime());
            assert!(sulid.datetime() <= dt);
            assert!(sulid.datetime() + Duration::from_millis(1) >= dt);
        }

        #[test]
        fn test_timestamp() {
            let dt = SystemTime::now();
            let sulid = Sulid::from_datetime(dt, 0, 0);
            let ts = dt
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            assert_eq!(u128::from(sulid.timestamp_ms()), ts);
        }

        #[test]
        fn default_is_nil() {
            assert_eq!(Sulid::default(), Sulid::nil());
        }

        #[test]
        fn nil_is_at_unix_epoch() {
            assert_eq!(Sulid::nil().datetime(), SystemTime::UNIX_EPOCH);
        }

        #[test]
        fn truncates_at_unix_epoch() {
            if let Some(before_epoch) = SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(100))
            {
                assert!(before_epoch < SystemTime::UNIX_EPOCH);
                assert_eq!(
                    Sulid::from_datetime(before_epoch, 0, 0).datetime(),
                    SystemTime::UNIX_EPOCH
                );
            } else {
                // Prior dates are not representable (e.g. wasm32-wasi)
            }
        }
    }
}

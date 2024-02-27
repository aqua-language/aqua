use crate::builtins::duration::Duration;
use crate::traits::DeepClone;

use num::Integer;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use time::format_description::well_known;

use super::string::String;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct Time(pub time::OffsetDateTime);

const EU: &[time::format_description::FormatItem<'_>] =
    time::macros::format_description!(version = 2, "[year]-[month]-[day] [hour]:[minute]:[second]");

const US: &[time::format_description::FormatItem<'_>] = time::macros::format_description!(
    version = 2,
    "[month]/[day]/[year] [hour]:[minute]:[second] [period case:upper]"
);

impl Serialize for Time {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let v = time::OffsetDateTime::format(self.0, &well_known::Iso8601::DEFAULT)
            .map_err(serde::ser::Error::custom)?;
        v.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Time, D::Error> {
        let s: std::string::String = Deserialize::deserialize(d)?;
        match time::PrimitiveDateTime::parse(s.as_ref(), &well_known::Iso8601::DEFAULT)
            .or_else(|_| time::PrimitiveDateTime::parse(s.as_ref(), &well_known::Rfc2822))
            .or_else(|_| time::PrimitiveDateTime::parse(s.as_ref(), &well_known::Rfc3339))
            .or_else(|_| time::PrimitiveDateTime::parse(s.as_ref(), &EU))
            .or_else(|_| time::PrimitiveDateTime::parse(s.as_ref(), &US))
            .map_err(serde::de::Error::custom)
        {
            Ok(v) => Ok(Time(v.assume_utc())),
            Err(e) => s
                .as_str()
                .parse()
                .ok()
                .and_then(|v| time::OffsetDateTime::from_unix_timestamp(v).ok())
                .map(Time)
                .ok_or(e),
        }
    }
}

impl DeepClone for Time {
    #[inline(always)]
    fn deep_clone(&self) -> Self {
        Time(self.0)
    }
}

impl Time {
    #[inline(always)]
    pub fn now() -> Time {
        Time(time::OffsetDateTime::now_utc())
    }

    #[inline(always)]
    pub fn from_seconds(seconds: i64) -> Time {
        Time(time::OffsetDateTime::from_unix_timestamp(seconds).unwrap())
    }

    #[inline(always)]
    pub fn from_milliseconds(millis: i128) -> Time {
        Self::from_nanoseconds(millis * 1000000)
    }

    #[inline(always)]
    pub fn from_microseconds(micros: i128) -> Time {
        Self::from_nanoseconds(micros * 1000)
    }

    #[inline(always)]
    pub fn from_nanoseconds(nanos: i128) -> Time {
        Time(time::OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap())
    }

    pub fn from_string(text: String, format: String) -> Time {
        let format = time::format_description::parse_owned::<2>(format.as_ref()).unwrap();
        Time(time::OffsetDateTime::parse(text.as_ref(), &format).unwrap())
    }

    pub const fn seconds(self) -> i64 {
        self.0.unix_timestamp()
    }

    pub const fn nanoseconds(self) -> i128 {
        self.0.unix_timestamp_nanos()
    }

    pub const fn milliseconds(self) -> i128 {
        self.0.unix_timestamp_nanos() / 1_000_000
    }

    pub const fn microseconds(self) -> i128 {
        self.0.unix_timestamp_nanos() / 1_000
    }

    pub const fn month(self) -> i32 {
        self.0.month() as i32
    }

    pub const fn day(self) -> i32 {
        self.0.day() as i32
    }

    pub const fn year(self) -> i32 {
        self.0.year()
    }

    pub fn to_text(self, format: String) -> String {
        let format = time::format_description::parse_owned::<2>(format.as_ref()).unwrap();
        String::from(self.0.format(&format).unwrap().as_str())
    }

    pub fn div_floor(self, duration: Duration) -> Self {
        Time::from_nanoseconds(Integer::div_floor(
            &self.nanoseconds(),
            &duration.nanoseconds(),
        ))
    }

    pub const fn zero() -> Self {
        Time(time::OffsetDateTime::UNIX_EPOCH)
    }
}

impl std::ops::Add<Duration> for Time {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Duration) -> Self::Output {
        Time::from_nanoseconds(self.nanoseconds() + rhs.nanoseconds())
    }
}

impl std::ops::Sub<Duration> for Time {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Duration) -> Self::Output {
        Time::from_nanoseconds(self.nanoseconds() - rhs.nanoseconds())
    }
}

impl std::ops::Mul<Duration> for Time {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Duration) -> Self::Output {
        Time::from_nanoseconds(self.nanoseconds() * rhs.nanoseconds())
    }
}

impl std::ops::Div<Duration> for Time {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Duration) -> Self::Output {
        Time::from_nanoseconds(self.nanoseconds() / rhs.nanoseconds())
    }
}

impl std::cmp::PartialEq<Duration> for Time {
    #[inline(always)]
    fn eq(&self, other: &Duration) -> bool {
        self.nanoseconds() == other.nanoseconds()
    }
}

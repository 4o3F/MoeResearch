use std::{borrow::Cow, fmt};

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use moe_research_error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit<T> {
    Limited(T),
    Unlimited,
}

impl<T> JsonSchema for Limit<T>
where
    T: JsonSchema,
{
    fn schema_name() -> Cow<'static, str> {
        "Limit".into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("{}::Limit", module_path!()).into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": ["integer", "null"],
            "minimum": -1,
            "description": "null or -1 means unlimited; non-negative integers are finite limits"
        })
    }
}

pub type CountLimit = Limit<usize>;
pub type DurationLimitMs = Limit<u64>;
pub type TokenLimit = Limit<u64>;

pub(crate) fn non_negative_integer_schema(_generator: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "integer",
        "minimum": 0
    })
}

pub(crate) fn optional_non_negative_integer_schema(_generator: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": ["integer", "null"],
        "minimum": 0
    })
}

pub(crate) fn optional_positive_integer_schema(_generator: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": ["integer", "null"],
        "minimum": 1
    })
}

impl<T> Limit<T> {
    pub const fn limited(value: T) -> Self {
        Self::Limited(value)
    }

    pub const fn unlimited() -> Self {
        Self::Unlimited
    }

    pub fn is_unlimited(self) -> bool
    where
        T: Copy,
    {
        matches!(self, Self::Unlimited)
    }
}

impl<T> Limit<T>
where
    T: Copy + Ord,
{
    pub fn exceeds(self, max: Self) -> bool {
        match (self, max) {
            (_, Self::Unlimited) => false,
            (Self::Unlimited, Self::Limited(_)) => true,
            (Self::Limited(value), Self::Limited(max)) => value > max,
        }
    }
}

impl<T> Limit<T>
where
    T: Copy + PartialEq + Default,
{
    pub fn is_zero(self) -> bool {
        matches!(self, Self::Limited(value) if value == T::default())
    }

    pub fn require_non_zero(self, field: &str) -> Result<()> {
        if self.is_zero() {
            return Err(Error::ConfigInvalid {
                message: format!("{field} must be greater than zero"),
            });
        }
        Ok(())
    }
}

impl Limit<usize> {
    pub fn permits_next(self, used: usize) -> bool {
        match self {
            Self::Unlimited => true,
            Self::Limited(max) => used < max,
        }
    }

    pub fn as_concurrency(self, natural_limit: usize) -> usize {
        match self {
            Self::Unlimited => natural_limit,
            Self::Limited(value) => value,
        }
    }

    /// Returns `true` if the supplied `used` value strictly exceeds this cap.
    ///
    /// Mirrors `Limit::permits_next` but expresses the post-increment
    /// perspective used by atomic counters (`fetch_add` returns the previous
    /// value; the caller checks `previous + 1 > max`).
    #[must_use]
    pub fn is_exceeded_by(self, used: usize) -> bool {
        match self {
            Self::Unlimited => false,
            Self::Limited(max) => used > max,
        }
    }

    /// Returns `true` if a `u64` atomic counter strictly exceeds this cap.
    ///
    /// Avoids narrowing the observed counter to `usize` before comparison,
    /// which would otherwise saturate on platforms where `usize` is narrower
    /// than `u64`. Used by the cross-aspect call counters whose backing
    /// store is `AtomicU64`.
    #[must_use]
    pub fn is_exceeded_by_u64(self, used: u64) -> bool {
        match self {
            Self::Unlimited => false,
            Self::Limited(max) => used > u64::try_from(max).unwrap_or(u64::MAX),
        }
    }
}

impl Limit<u64> {
    pub fn is_elapsed(self, elapsed_ms: u64) -> bool {
        match self {
            Self::Unlimited => false,
            Self::Limited(limit_ms) => elapsed_ms >= limit_ms,
        }
    }

    /// Returns `true` if the supplied `used` total strictly exceeds this cap.
    ///
    /// Used by the cross-aspect token guard: providers report cumulative
    /// usage as `u64`, and we want to reject the call that pushes the total
    /// past the configured ceiling.
    #[must_use]
    pub fn is_exceeded_by_u64(self, used: u64) -> bool {
        match self {
            Self::Unlimited => false,
            Self::Limited(max) => used > max,
        }
    }

    /// Returns `true` if the supplied `used` total has reached or surpassed
    /// this cap.
    ///
    /// Used before dispatch for non-reservable resources such as tokens, where
    /// once no capacity remains a provider call must not be issued. Unlike
    /// [`Self::is_exceeded_by_u64`], the boundary value `used == max` is also
    /// rejected because tokens cannot be "spent and undone" the way an atomic
    /// counter can be rolled back.
    #[must_use]
    pub fn is_exhausted_by_u64(self, used: u64) -> bool {
        match self {
            Self::Unlimited => false,
            Self::Limited(max) => used >= max,
        }
    }
}

impl<T> Serialize for Limit<T>
where
    T: Copy + TryInto<u64>,
    T::Error: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Unlimited => serializer.serialize_i8(-1),
            Self::Limited(value) => {
                serializer.serialize_u64((*value).try_into().map_err(serde::ser::Error::custom)?)
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for Limit<T>
where
    T: TryFrom<i128>,
    T::Error: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = deserializer.deserialize_any(LimitVisitor)?;
        match value {
            None | Some(-1) => Ok(Self::Unlimited),
            Some(0..) => T::try_from(value.expect("checked value"))
                .map(Self::Limited)
                .map_err(de::Error::custom),
            Some(_) => Err(de::Error::custom("budget limit must be -1 or non-negative")),
        }
    }
}

struct LimitVisitor;

impl Visitor<'_> for LimitVisitor {
    type Value = Option<i128>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("null, -1, or a non-negative integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Some(i128::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Some(i128::from(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }
}

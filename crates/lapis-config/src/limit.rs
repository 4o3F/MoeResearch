use std::{borrow::Cow, fmt};

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use lapis_error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigLimit<T> {
    Limited(T),
    Unlimited,
}

pub type CountLimit = ConfigLimit<usize>;
pub type DurationLimitMs = ConfigLimit<u64>;
pub type TokenLimit = ConfigLimit<u64>;

impl<T> ConfigLimit<T> {
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

impl<T> ConfigLimit<T>
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

impl<T> ConfigLimit<T>
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

impl<T> JsonSchema for ConfigLimit<T>
where
    T: JsonSchema,
{
    fn schema_name() -> Cow<'static, str> {
        "ConfigLimit".into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("{}::ConfigLimit", module_path!()).into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": ["integer", "null"],
            "minimum": -1,
            "description": "null or -1 means unlimited; non-negative integers are finite limits"
        })
    }
}

impl<T> Serialize for ConfigLimit<T>
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

impl<'de, T> Deserialize<'de> for ConfigLimit<T>
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

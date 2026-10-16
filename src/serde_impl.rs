//! Serde support (the `serde` feature): data URLs and MIME types are serialized as strings,
//! and deserialized by parsing those strings.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::{DataUrl, Error, MimeType};

/// Serialized as its string form (the same as `to_string()`, written without building a string first
/// by serializers that support that, such as serde_json's).
impl Serialize for DataUrl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Deserialized from a string with [`DataUrl::parse()`]; invalid data URLs are an error.
impl<'de> Deserialize<'de> for DataUrl {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ParsingVisitor::new("a data URL"))
    }
}

/// Serialized as its string form (the same as `to_string()`).
impl Serialize for MimeType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Deserialized from a string with [`MimeType::parse()`]; invalid MIME types are an error.
impl<'de> Deserialize<'de> for MimeType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ParsingVisitor::new("a MIME type"))
    }
}

struct ParsingVisitor<T> {
    expecting: &'static str,
    marker: PhantomData<T>,
}

impl<T> ParsingVisitor<T> {
    fn new(expecting: &'static str) -> Self {
        ParsingVisitor {
            expecting,
            marker: PhantomData,
        }
    }
}

impl<T: FromStr<Err = Error>> Visitor<'_> for ParsingVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.expecting)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<T, E> {
        // The input isn't included in the error, since data URLs can be huge
        value
            .parse()
            .map_err(|error| E::custom(format_args!("expected {}: {error}", self.expecting)))
    }
}

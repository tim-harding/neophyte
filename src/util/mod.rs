mod parse;
pub mod vec2;

pub use parse::Parse;

use rmpv::Value;
use std::{
    fmt::{Debug, DebugStruct},
    vec::IntoIter,
};

pub trait MaybeInto<T>: Sized {
    fn maybe_into(self) -> Option<T>;
}

impl<T: Parse> MaybeInto<T> for Value {
    fn maybe_into(self) -> Option<T> {
        T::parse(self)
    }
}

pub fn parse_map(value: Value) -> Option<Vec<(Value, Value)>> {
    match value {
        Value::Map(map) => Some(map),
        _ => None,
    }
}

/// Used for positive integer values where -1 is a sentinel. The sentinel is
/// represented by None.
pub fn parse_maybe_u64(value: Value) -> Option<Option<u64>> {
    match value {
        Value::Integer(i) => Some(i.as_u64()),
        _ => None,
    }
}

/// Takes the first element from an array value
pub fn parse_first_element(value: Value) -> Option<Value> {
    Values::new(value)?.next()
}

/// Add a field to the debug struct if it is not None
pub fn maybe_field<T: Debug>(s: &mut DebugStruct, name: &str, field: Option<T>) {
    if let Some(t) = field {
        s.field(name, &t);
    }
}

/// Add a an array field to the debug struct if it is not empty
pub fn maybe_other_field(s: &mut DebugStruct, field: &[(String, Value)]) {
    if !field.is_empty() {
        s.field("other", &field);
    }
}

pub struct Values(IntoIter<Value>);

impl Values {
    pub fn new(value: Value) -> Option<Self> {
        match value {
            Value::Array(array) => Some(Self(array.into_iter())),
            _ => None,
        }
    }

    pub fn next<T: Parse>(&mut self) -> Option<T> {
        T::parse(self.0.next()?)
    }

    pub fn into_inner(self) -> IntoIter<Value> {
        self.0
    }

    pub fn map<T: Parse>(self) -> Option<Vec<T>> {
        self.into_inner().map(T::parse).collect()
    }
}

pub fn srgb(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c < 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

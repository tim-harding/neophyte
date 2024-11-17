use rmpv::Value;
use std::{
    fmt::{Debug, DebugStruct},
    vec::IntoIter,
};

use crate::values::Values;

/// Construct the given type from a MessagePack value. Similar to TryFrom.
pub trait Parse: Sized {
    fn parse(value: Value) -> Option<Self>;
}

impl Parse for bool {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::Boolean(b) => Some(b),
            _ => None,
        }
    }
}

impl Parse for String {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::String(s) => s.into_str(),
            _ => None,
        }
    }
}

impl Parse for char {
    fn parse(value: Value) -> Option<Self> {
        String::parse(value)?.chars().next()
    }
}

impl Parse for u64 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::Integer(n) => n.as_u64(),
            _ => None,
        }
    }
}

impl Parse for u32 {
    fn parse(value: Value) -> Option<Self> {
        u64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for u16 {
    fn parse(value: Value) -> Option<Self> {
        u64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for u8 {
    fn parse(value: Value) -> Option<Self> {
        u64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for i64 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::Integer(n) => n.as_i64(),
            _ => None,
        }
    }
}

impl Parse for i32 {
    fn parse(value: Value) -> Option<Self> {
        i64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for i16 {
    fn parse(value: Value) -> Option<Self> {
        i64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for i8 {
    fn parse(value: Value) -> Option<Self> {
        i64::parse(value).and_then(|n| n.try_into().ok())
    }
}

impl Parse for f64 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::F64(n) => Some(n),
            Value::F32(n) => Some(n as f64),
            Value::Integer(n) => n.as_f64(),
            _ => None,
        }
    }
}

impl Parse for f32 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::F32(n) => Some(n),
            Value::F64(n) => Some(n as f32),
            Value::Integer(n) => n.as_f64().map(|n| n as f32),
            _ => None,
        }
    }
}

impl Parse for Value {
    fn parse(value: Value) -> Option<Self> {
        Some(value)
    }
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(value: Value) -> Option<Self> {
        Values::new(value)?.map()
    }
}

impl<T: Parse> Parse for Option<T> {
    fn parse(value: Value) -> Option<Self> {
        Some(T::parse(value))
    }
}

/// Like TryInto but with Option as the return type.
pub trait MaybeInto<T>: Sized {
    fn maybe_into(self) -> Option<T>;
}

impl<T: Parse> MaybeInto<T> for Value {
    fn maybe_into(self) -> Option<T> {
        T::parse(self)
    }
}

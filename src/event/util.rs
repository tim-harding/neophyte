use nvim_rs::Value;
use std::{
    fmt::{Debug, DebugStruct},
    vec::IntoIter,
};

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

impl Parse for u64 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::Integer(n) => n.as_u64(),
            _ => None,
        }
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

impl Parse for f64 {
    fn parse(value: Value) -> Option<Self> {
        match value {
            Value::F64(n) => Some(n),
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
        map_array(value, T::parse)
    }
}

impl<T: Parse> Parse for Option<T> {
    fn parse(value: Value) -> Option<Self> {
        Some(T::parse(value))
    }
}

pub trait MaybeFrom<T>: Sized {
    fn maybe_from(value: T) -> Option<Self>;
}

impl<T> MaybeFrom<Value> for T
where
    T: Parse,
{
    fn maybe_from(value: Value) -> Option<Self> {
        Self::parse(value)
    }
}

pub trait MaybeInto<T>: Sized {
    fn maybe_into(self) -> Option<T>;
}

impl<T, U> MaybeInto<U> for T
where
    U: MaybeFrom<T>,
{
    fn maybe_into(self) -> Option<U> {
        U::maybe_from(self)
    }
}

pub fn map_array<T>(value: Value, f: fn(Value) -> Option<T>) -> Option<Vec<T>> {
    Vec::parse(value)?.into_iter().map(f).collect()
}

pub fn parse_map(value: Value) -> Option<Vec<(Value, Value)>> {
    match value {
        Value::Map(map) => Some(map),
        _ => None,
    }
}

pub fn parse_ext(value: Value, expected_type: i8) -> Option<Vec<u8>> {
    match value {
        Value::Ext(type_id, data) => (type_id == expected_type).then_some(data),
        _ => None,
    }
}

pub fn maybe_field<T: Debug>(s: &mut DebugStruct, name: &str, field: Option<T>) {
    if let Some(t) = field {
        s.field(name, &t);
    }
}

pub fn maybe_other_field(s: &mut DebugStruct, field: &[(String, Value)]) {
    if !field.is_empty() {
        s.field("other", &field);
    }
}

pub struct Values(IntoIter<Value>);

impl Values {
    pub fn new(value: Value) -> Option<Self> {
        Some(Self(Vec::parse(value)?.into_iter()))
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

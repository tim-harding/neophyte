mod parse;

pub use parse::Parse;

use nvim_rs::Value;
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

    pub fn map_with<T: Parse>(self, f: impl Fn(Value) -> Option<Value>) -> Option<Vec<T>> {
        self.into_inner().map(|v| T::parse(f(v)?)).collect()
    }
}
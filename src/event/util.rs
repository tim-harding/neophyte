use nvim_rs::Value;
use std::{
    fmt::{Debug, DebugStruct},
    vec::IntoIter,
};

pub trait Parse {
    fn parse(value: Value) -> Option<Self>
    where
        Self: Sized;
}

pub fn parse_bool(value: Value) -> Option<bool> {
    match value {
        Value::Boolean(b) => Some(b),
        _ => None,
    }
}

pub fn parse_string(value: Value) -> Option<String> {
    match value {
        Value::String(s) => s.into_str(),
        _ => None,
    }
}

pub fn parse_u64(value: Value) -> Option<u64> {
    match value {
        Value::Integer(n) => n.as_u64(),
        _ => None,
    }
}

pub fn parse_i64(value: Value) -> Option<i64> {
    match value {
        Value::Integer(n) => n.as_i64(),
        _ => None,
    }
}

pub fn parse_f64(value: Value) -> Option<f64> {
    match value {
        Value::F64(n) => Some(n),
        _ => None,
    }
}

pub fn parse_array(value: Value) -> Option<Vec<Value>> {
    match value {
        Value::Array(array) => Some(array),
        _ => None,
    }
}

pub fn map_array<T>(value: Value, f: fn(Value) -> Option<T>) -> Option<Vec<T>> {
    parse_array(value)?.into_iter().map(f).collect()
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

pub struct ValueIter(IntoIter<Value>);

impl ValueIter {
    pub fn new(value: Value) -> Option<Self> {
        Some(Self(parse_array(value)?.into_iter()))
    }

    pub fn next(&mut self) -> Option<Value> {
        self.0.next()
    }

    pub fn next_bool(&mut self) -> Option<bool> {
        parse_bool(self.next()?)
    }

    pub fn next_string(&mut self) -> Option<String> {
        parse_string(self.next()?)
    }

    pub fn next_u64(&mut self) -> Option<u64> {
        parse_u64(self.next()?)
    }

    pub fn next_i64(&mut self) -> Option<i64> {
        parse_i64(self.next()?)
    }

    pub fn next_f64(&mut self) -> Option<f64> {
        parse_f64(self.next()?)
    }

    pub fn next_ext(&mut self, expected_type: i8) -> Option<Vec<u8>> {
        parse_ext(self.next()?, expected_type)
    }

    pub fn next_map(&mut self) -> Option<Vec<(Value, Value)>> {
        parse_map(self.next()?)
    }

    pub fn next_parse<P: Parse>(&mut self) -> Option<P> {
        P::parse(self.next()?)
    }
}

fn test(value: Value) -> Option<()> {
    let mut iter = ValueIter::new(value)?;
    let b = iter.next_bool()?;
    let t: Test = iter.next_parse()?;
    Some(())
}

struct Test;

impl Parse for Test {
    fn parse(value: Value) -> Option<Self> {
        Some(Self)
    }
}

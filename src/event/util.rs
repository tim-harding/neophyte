use nvim_rs::Value;
use std::fmt::{Debug, DebugStruct};

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

pub fn parse_array(value: Value) -> Option<Vec<Value>> {
    match value {
        Value::Array(array) => Some(array),
        _ => None,
    }
}

pub fn parse_map(value: Value) -> Option<Vec<(Value, Value)>> {
    match value {
        Value::Map(map) => Some(map),
        _ => None,
    }
}

pub fn maybe_field<T: Debug>(s: &mut DebugStruct, name: &str, field: Option<T>) {
    if let Some(t) = field {
        s.field(name, &t);
    }
}

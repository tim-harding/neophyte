use crate::Values;
use rmpv::Value;
use std::fmt::{Debug, DebugStruct};

/// Gets the inner map type from a MessagePack value.
pub fn parse_map(value: Value) -> Option<Vec<(Value, Value)>> {
    match value {
        Value::Map(map) => Some(map),
        _ => None,
    }
}

/// Used for positive integer values where -1 is a sentinel. The sentinel is
/// represented by None.
pub fn parse_maybe_u32(value: Value) -> Option<Option<u32>> {
    match value {
        Value::Integer(i) => Some(i.as_u64().and_then(|i| i.try_into().ok())),
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

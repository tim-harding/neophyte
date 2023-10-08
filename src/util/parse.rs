use super::Values;
use rmpv::Value;

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
            _ => None,
        }
    }
}

impl Parse for f32 {
    fn parse(value: Value) -> Option<Self> {
        f64::parse(value).map(|n| n as f32)
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

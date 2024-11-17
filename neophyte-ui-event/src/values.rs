use rmpv::Value;
use std::vec::IntoIter;

use crate::Parse;

/// An iterator over values in a MessagePack array.
pub struct Values(IntoIter<Value>);

impl Values {
    /// Attempt to construct an iterator over the given array value.
    pub fn new(value: Value) -> Option<Self> {
        match value {
            Value::Array(array) => Some(Self(array.into_iter())),
            _ => None,
        }
    }

    /// Get the next value and convert it to the destination type.
    #[allow(clippy::should_implement_trait)]
    pub fn next<T: Parse>(&mut self) -> Option<T> {
        T::parse(self.0.next()?)
    }

    /// Get the internal value iterator.
    pub fn into_inner(self) -> IntoIter<Value> {
        self.0
    }

    /// Try to convert the entire iterator to the given type.
    pub fn map<T: Parse>(self) -> Option<Vec<T>> {
        self.into_inner().map(T::parse).collect()
    }
}

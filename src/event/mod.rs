mod grid_resize;
mod option_set;
mod set_title;
mod util;

use self::{
    grid_resize::{GridResize, GridResizeParseError},
    option_set::{OptionSet, OptionSetParseError},
    set_title::{SetTitle, SetTitleParseError},
};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    SetTitle(SetTitle),
    OptionSet(OptionSet),
}

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter();
                let event_name = iter.next().ok_or(EventParseError::Malformed)?;
                match event_name {
                    Value::String(s) => match s.as_str() {
                        Some(s) => match s {
                            "grid_resize" => Ok(Self::GridResize(GridResize::try_from(
                                iter.next().ok_or(EventParseError::Malformed)?,
                            )?)),
                            "set_title" => Ok(Self::SetTitle(SetTitle::try_from(
                                iter.next().ok_or(EventParseError::Malformed)?,
                            )?)),
                            "option_set" => Ok(Self::OptionSet(OptionSet::try_from(iter)?)),
                            _ => Err(EventParseError::UnknownEvent(s.to_string())),
                        },
                        None => Err(EventParseError::Malformed),
                    },
                    _ => Err(EventParseError::Malformed),
                }
            }
            _ => Err(EventParseError::Malformed),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EventParseError {
    #[error("Event is malformed")]
    Malformed,
    #[error("Received an unrecognized event name: {0}")]
    UnknownEvent(String),
    #[error("{0}")]
    GridResize(#[from] GridResizeParseError),
    #[error("{0}")]
    SetTitle(#[from] SetTitleParseError),
    #[error("{0}")]
    OptionSet(#[from] OptionSetParseError),
}

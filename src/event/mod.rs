mod default_colors_set;
mod grid_clear;
mod grid_destroy;
mod grid_resize;
mod option_set;
mod set_title;
mod util;

use self::{
    default_colors_set::{DefaultColorsSet, DefaultColorsSetParseError},
    grid_clear::{GridClear, GridClearParseError},
    grid_destroy::{GridDestroy, GridDestroyParseError},
    grid_resize::{GridResize, GridResizeParseError},
    option_set::{OptionSet, OptionSetParseError},
    set_title::{SetTitle, SetTitleParseError},
    util::parse_array,
};
use crate::event::util::parse_string;
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    SetTitle(SetTitle),
    OptionSet(OptionSet),
    GridClear(GridClear),
    GridDestroy(GridDestroy),
    DefaultColorsSet(DefaultColorsSet),
}

macro_rules! event_from {
    ($x:ident) => {
        impl From<$x> for Event {
            fn from(value: $x) -> Self {
                Self::$x(value)
            }
        }
    };
}

event_from!(GridResize);
event_from!(SetTitle);
event_from!(OptionSet);
event_from!(GridClear);
event_from!(GridDestroy);
event_from!(DefaultColorsSet);

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let array = parse_array(value).ok_or(EventParseError::Malformed)?;
        let mut iter = array.into_iter();
        let event_name = iter.next().ok_or(EventParseError::Malformed)?;
        fn try_next(mut iter: IntoIter<Value>) -> Result<Value, EventParseError> {
            iter.next().ok_or(EventParseError::Malformed)
        }
        let event_name = parse_string(event_name).ok_or(EventParseError::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => Ok(GridResize::try_from(try_next(iter)?)?.into()),
            "set_title" => Ok(SetTitle::try_from(try_next(iter)?)?.into()),
            "option_set" => Ok(OptionSet::try_from(iter)?.into()),
            "grid_clear" => Ok(GridClear::try_from(try_next(iter)?)?.into()),
            "grid_destroy" => Ok(GridDestroy::try_from(try_next(iter)?)?.into()),
            "default_colors_set" => Ok(DefaultColorsSet::try_from(try_next(iter)?)?.into()),
            _ => Err(EventParseError::UnknownEvent(event_name)),
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
    #[error("{0}")]
    GridClear(#[from] GridClearParseError),
    #[error("{0}")]
    GridDestroy(#[from] GridDestroyParseError),
    #[error("{0}")]
    DefaultColorsSet(#[from] DefaultColorsSetParseError),
}

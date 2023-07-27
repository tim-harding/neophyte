mod option_set;
mod util;

use self::{
    option_set::{OptionSet, OptionSetParseError},
    util::parse_u64,
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

#[derive(Debug, Clone, Copy)]
pub struct GridResize {
    pub grid: u64,
    pub width: u64,
    pub height: u64,
}

impl TryFrom<Value> for GridResize {
    type Error = GridResizeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter().map(parse_u64).flatten();
                let grid = iter.next().ok_or(GridResizeParseError)?;
                let width = iter.next().ok_or(GridResizeParseError)?;
                let height = iter.next().ok_or(GridResizeParseError)?;
                Ok(Self {
                    grid,
                    width,
                    height,
                })
            }
            _ => Err(GridResizeParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_resize event")]
pub struct GridResizeParseError;

#[derive(Debug, Clone)]
pub struct SetTitle(Vec<String>);

impl TryFrom<Value> for SetTitle {
    type Error = SetTitleParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let titles: Option<Vec<_>> = match value {
            Value::Array(array) => array
                .into_iter()
                .map(|value| match value {
                    Value::String(s) => s.into_str(),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        let titles = titles.ok_or(SetTitleParseError)?;
        Ok(Self(titles))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse set_title event")]
pub struct SetTitleParseError;

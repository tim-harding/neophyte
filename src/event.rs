use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    GridResize(GridResize),
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
}

#[derive(Debug, Clone, Copy)]
struct GridResize {
    pub grid: u64,
    pub width: u64,
    pub height: u64,
}

impl TryFrom<Value> for GridResize {
    type Error = GridResizeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter().map(pos_int).flatten();
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

fn pos_int(value: Value) -> Option<u64> {
    match value {
        Value::Integer(n) => n.as_u64(),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_resize event")]
struct GridResizeParseError;

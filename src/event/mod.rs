mod default_colors_set;
mod grid_clear;
mod grid_cursor_goto;
mod grid_destroy;
mod grid_line;
mod grid_resize;
mod grid_scroll;
mod hl_attr_define;
mod hl_group_set;
mod mode_change;
mod mode_info_set;
mod option_set;
mod set_icon;
mod set_title;
mod util;

use self::{
    default_colors_set::DefaultColorsSet, grid_clear::GridClear, grid_cursor_goto::GridCursorGoto,
    grid_destroy::GridDestroy, grid_line::GridLine, grid_resize::GridResize,
    grid_scroll::GridScroll, hl_attr_define::HlAttrDefine, hl_group_set::HlGroupSet,
    mode_change::ModeChange, mode_info_set::ModeInfoSet, option_set::OptionSet, set_icon::SetIcon,
    set_title::SetTitle, util::parse_array,
};
use crate::event::util::parse_string;
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    SetTitle(SetTitle),
    SetIcon(SetIcon),
    OptionSet(Vec<OptionSet>),
    GridClear(GridClear),
    GridDestroy(GridDestroy),
    DefaultColorsSet(DefaultColorsSet),
    HlAttrDefine(Vec<HlAttrDefine>),
    ModeChange(ModeChange),
    ModeInfoSet(Vec<ModeInfoSet>),
    HlGroupSet(Vec<HlGroupSet>),
    GridCursorGoto(GridCursorGoto),
    GridScroll(GridScroll),
    GridLine(Vec<GridLine>),
    Clear,
    EolClear,
    MouseOn,
    MouseOff,
    BusyStart,
    BusyStop,
    Suspend,
    UpdateMenu,
    Bell,
    VisualBell,
    Flush,
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

macro_rules! event_from_vec {
    ($x:ident) => {
        impl From<Vec<$x>> for Event {
            fn from(value: Vec<$x>) -> Self {
                Self::$x(value)
            }
        }
    };
}

event_from!(GridResize);
event_from!(SetTitle);
event_from!(SetIcon);
event_from_vec!(OptionSet);
event_from!(GridClear);
event_from!(GridDestroy);
event_from!(DefaultColorsSet);
event_from_vec!(HlAttrDefine);
event_from!(ModeChange);
event_from_vec!(ModeInfoSet);
event_from_vec!(HlGroupSet);
event_from!(GridCursorGoto);
event_from!(GridScroll);
event_from_vec!(GridLine);

fn single<T: Into<Event>>(
    mut iter: IntoIter<Value>,
    f: fn(Value) -> Option<T>,
    e: EventParseError,
) -> Result<Event, EventParseError> {
    let next = iter.next().ok_or(EventParseError::Malformed)?;
    Ok(f(next).ok_or(e)?.into())
}

fn multi<T>(
    iter: IntoIter<Value>,
    f: fn(Value) -> Option<T>,
    e: EventParseError,
) -> Result<Event, EventParseError>
where
    Vec<T>: Into<Event>,
{
    let mapped: Option<Vec<_>> = iter.map(f).collect();
    Ok(mapped.ok_or(e)?.into())
}

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let array = parse_array(value).ok_or(EventParseError::Malformed)?;
        let mut iter = array.into_iter();
        let event_name = iter.next().ok_or(EventParseError::Malformed)?;
        let event_name = parse_string(event_name).ok_or(EventParseError::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => single(iter, GridResize::parse, EventParseError::GridResize),
            "set_title" => single(iter, SetTitle::parse, EventParseError::SetTitle),
            "set_icon" => single(iter, SetIcon::parse, EventParseError::SetIcon),
            "option_set" => multi(iter, OptionSet::parse, EventParseError::OptionSet),
            "grid_clear" => single(iter, GridClear::parse, EventParseError::GridClear),
            "grid_destroy" => single(iter, GridDestroy::parse, EventParseError::GridDestroy),
            "default_colors_set" => single(
                iter,
                DefaultColorsSet::parse,
                EventParseError::DefaultColorsSet,
            ),
            "hl_attr_define" => multi(iter, HlAttrDefine::parse, EventParseError::HlAttrDefine),
            "mode_change" => single(iter, ModeChange::parse, EventParseError::ModeChange),
            "mode_info_set" => multi(iter, ModeInfoSet::parse, EventParseError::ModeInfoSet),
            "hl_group_set" => multi(iter, HlGroupSet::parse, EventParseError::HlGroupSet),
            "grid_cursor_goto" => {
                single(iter, GridCursorGoto::parse, EventParseError::GridCursorGoto)
            }
            "grid_scroll" => single(iter, GridScroll::parse, EventParseError::GridScroll),
            "grid_line" => multi(iter, GridLine::parse, EventParseError::GridLine),
            "clear" => Ok(Self::Clear),
            "eol_clear" => Ok(Self::EolClear),
            "mouse_on" => Ok(Self::MouseOn),
            "mouse_off" => Ok(Self::MouseOff),
            "busy_start" => Ok(Self::BusyStart),
            "busy_stop" => Ok(Self::BusyStop),
            "suspend" => Ok(Self::Suspend),
            "update_menu" => Ok(Self::UpdateMenu),
            "bell" => Ok(Self::Bell),
            "visual_bell" => Ok(Self::VisualBell),
            "flush" => Ok(Self::Flush),
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
    #[error("Failed to parse grid_resize event")]
    GridResize,
    #[error("Failed to parse set_title event")]
    SetTitle,
    #[error("Failed to parse set_icon event")]
    SetIcon,
    #[error("Failed to parse option_set event")]
    OptionSet,
    #[error("Failed to parse grid_clear event")]
    GridClear,
    #[error("Failed to parse grid_destroy event")]
    GridDestroy,
    #[error("Failed to parse default_colors_set event")]
    DefaultColorsSet,
    #[error("Failed to parse hl_attr_define event")]
    HlAttrDefine,
    #[error("Failed to parse mode_change event")]
    ModeChange,
    #[error("Failed to parse mode_info_set event")]
    ModeInfoSet,
    #[error("Failed to parse hl_group_set event")]
    HlGroupSet,
    #[error("Failed to parse grid_cursor_goto event")]
    GridCursorGoto,
    #[error("Failed to parse grid_scroll event")]
    GridScroll,
    #[error("Failed to parse grid_line event")]
    GridLine,
}

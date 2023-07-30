mod default_colors_set;
mod grid_cursor_goto;
mod grid_line;
mod grid_resize;
mod grid_scroll;
mod hl_attr_define;
mod hl_group_set;
mod mode_change;
mod mode_info_set;
mod option_set;
mod util;

use self::{
    default_colors_set::DefaultColorsSet,
    grid_cursor_goto::GridCursorGoto,
    grid_line::GridLine,
    grid_resize::GridResize,
    grid_scroll::GridScroll,
    hl_attr_define::HlAttrDefine,
    hl_group_set::HlGroupSet,
    mode_change::ModeChange,
    mode_info_set::ModeInfoSet,
    option_set::OptionSet,
    util::{parse_array, parse_u64},
};
use crate::event::util::parse_string;
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    SetTitle(Vec<String>),
    SetIcon(Vec<String>),
    OptionSet(Vec<OptionSet>),
    GridClear(Vec<u64>),
    GridDestroy(Vec<u64>),
    DefaultColorsSet(DefaultColorsSet),
    HlAttrDefine(Vec<HlAttrDefine>),
    ModeChange(ModeChange),
    ModeInfoSet(ModeInfoSet),
    HlGroupSet(HlGroupSet),
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
event_from_vec!(OptionSet);
event_from!(DefaultColorsSet);
event_from_vec!(HlAttrDefine);
event_from!(ModeChange);
event_from!(ModeInfoSet);
event_from!(HlGroupSet);
event_from!(GridCursorGoto);
event_from!(GridScroll);
event_from_vec!(GridLine);

fn single<T: Into<Event>>(
    mut iter: IntoIter<Value>,
    f: fn(Value) -> Option<T>,
    e: Error,
) -> Result<Event, Error> {
    let next = iter.next().ok_or(Error::Malformed)?;
    f(next).ok_or(e).map(Into::into)
}

fn multi<T>(iter: IntoIter<Value>, f: fn(Value) -> Option<T>, e: Error) -> Result<Event, Error>
where
    Vec<T>: Into<Event>,
{
    let mapped: Option<Vec<_>> = iter.map(f).collect();
    mapped.ok_or(e).map(Into::into)
}

fn multi_u64(iter: IntoIter<Value>, f: fn(Vec<u64>) -> Event, e: Error) -> Result<Event, Error> {
    let grids: Option<Vec<_>> = iter.map(parse_u64).collect();
    grids.ok_or(e).map(f)
}

fn multi_string(
    iter: IntoIter<Value>,
    f: fn(Vec<String>) -> Event,
    e: Error,
) -> Result<Event, Error> {
    let grids: Option<Vec<_>> = iter.map(parse_string).collect();
    grids.ok_or(e).map(f)
}

impl TryFrom<Value> for Event {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let array = parse_array(value).ok_or(Error::Malformed)?;
        let mut iter = array.into_iter();
        let event_name = iter.next().ok_or(Error::Malformed)?;
        let event_name = parse_string(event_name).ok_or(Error::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => single(iter, GridResize::parse, Error::GridResize),
            "set_title" => multi_string(iter, Self::SetTitle, Error::SetTitle),
            "set_icon" => multi_string(iter, Self::SetIcon, Error::SetIcon),
            "option_set" => multi(iter, OptionSet::parse, Error::OptionSet),
            "grid_clear" => multi_u64(iter, Event::GridClear, Error::GridClear),
            "grid_destroy" => multi_u64(iter, Event::GridDestroy, Error::GridDestroy),
            "default_colors_set" => single(iter, DefaultColorsSet::parse, Error::DefaultColorsSet),
            "hl_attr_define" => multi(iter, HlAttrDefine::parse, Error::HlAttrDefine),
            "mode_change" => single(iter, ModeChange::parse, Error::ModeChange),
            "mode_info_set" => single(iter, ModeInfoSet::parse, Error::ModeInfoSet),
            "hl_group_set" => single(iter, HlGroupSet::parse, Error::HlGroupSet),
            "grid_cursor_goto" => single(iter, GridCursorGoto::parse, Error::GridCursorGoto),
            "grid_scroll" => single(iter, GridScroll::parse, Error::GridScroll),
            "grid_line" => multi(iter, GridLine::parse, Error::GridLine),
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
            _ => Err(Error::UnknownEvent(event_name)),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
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

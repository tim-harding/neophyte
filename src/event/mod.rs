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
mod tabline_update;
mod types;
mod util;
mod win_viewport;

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
    tabline_update::TablineUpdate,
    types::MessageContent,
    util::{parse_array, parse_u64},
    win_viewport::WinViewport,
};
use crate::event::util::parse_string;
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(Vec<GridResize>),
    SetTitle(Vec<String>),
    SetIcon(Vec<String>),
    OptionSet(Vec<OptionSet>),
    GridClear(Vec<u64>),
    GridDestroy(Vec<u64>),
    DefaultColorsSet(Vec<DefaultColorsSet>),
    HlAttrDefine(Vec<HlAttrDefine>),
    ModeChange(Vec<ModeChange>),
    ModeInfoSet(Vec<ModeInfoSet>),
    HlGroupSet(Vec<HlGroupSet>),
    GridCursorGoto(Vec<GridCursorGoto>),
    GridScroll(Vec<GridScroll>),
    GridLine(Vec<GridLine>),
    WinViewport(Vec<WinViewport>),
    TablineUpdate(Vec<TablineUpdate>),
    MsgShowmode(Vec<MessageContent>),
    MsgShowcmd(Vec<MessageContent>),
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
    CmdlineHide,
}

macro_rules! event_from {
    ($x:ident) => {
        impl From<Vec<$x>> for Event {
            fn from(value: Vec<$x>) -> Self {
                Self::$x(value)
            }
        }
    };
}

event_from!(GridResize);
event_from!(OptionSet);
event_from!(DefaultColorsSet);
event_from!(HlAttrDefine);
event_from!(ModeChange);
event_from!(ModeInfoSet);
event_from!(HlGroupSet);
event_from!(GridCursorGoto);
event_from!(GridScroll);
event_from!(GridLine);
event_from!(WinViewport);
event_from!(TablineUpdate);

fn unique<T>(
    iter: IntoIter<Value>,
    parser: fn(Value) -> Option<T>,
    error: Error,
) -> Result<Event, Error>
where
    Vec<T>: Into<Event>,
{
    let mapped: Option<Vec<_>> = iter.map(parser).collect();
    mapped.ok_or(error).map(Into::into)
}

fn shared<T>(
    iter: IntoIter<Value>,
    parser: fn(Value) -> Option<T>,
    event_variant: fn(Vec<T>) -> Event,
    error: Error,
) -> Result<Event, Error> {
    let events: Option<Vec<T>> = iter
        .map(|v| parser(parse_array(v)?.into_iter().next()?))
        .collect();
    events.map(event_variant).ok_or(error)
}

impl TryFrom<Value> for Event {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let array = parse_array(value).ok_or(Error::Malformed)?;
        let mut iter = array.into_iter();
        let event_name = iter.next().ok_or(Error::Malformed)?;
        let event_name = parse_string(event_name).ok_or(Error::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => unique(iter, GridResize::parse, Error::GridResize),
            "set_title" => shared(iter, parse_string, Self::SetTitle, Error::SetTitle),
            "set_icon" => shared(iter, parse_string, Self::SetIcon, Error::SetIcon),
            "option_set" => unique(iter, OptionSet::parse, Error::OptionSet),
            "grid_clear" => shared(iter, parse_u64, Event::GridClear, Error::GridClear),
            "grid_destroy" => shared(iter, parse_u64, Event::GridDestroy, Error::GridDestroy),
            "default_colors_set" => unique(iter, DefaultColorsSet::parse, Error::DefaultColorsSet),
            "hl_attr_define" => unique(iter, HlAttrDefine::parse, Error::HlAttrDefine),
            "mode_change" => unique(iter, ModeChange::parse, Error::ModeChange),
            "mode_info_set" => unique(iter, ModeInfoSet::parse, Error::ModeInfoSet),
            "hl_group_set" => unique(iter, HlGroupSet::parse, Error::HlGroupSet),
            "grid_cursor_goto" => unique(iter, GridCursorGoto::parse, Error::GridCursorGoto),
            "grid_scroll" => unique(iter, GridScroll::parse, Error::GridScroll),
            "grid_line" => unique(iter, GridLine::parse, Error::GridLine),
            "win_viewport" => unique(iter, WinViewport::parse, Error::WinViewport),
            "tabline_update" => unique(iter, TablineUpdate::parse, Error::TablineUpdate),
            "msg_showmode" => shared(
                iter,
                MessageContent::parse,
                Self::MsgShowmode,
                Error::MsgShowmode,
            ),
            "msg_showcmd" => shared(
                iter,
                MessageContent::parse,
                Self::MsgShowcmd,
                Error::MsgShowcmd,
            ),
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
            // TODO: This event receives an undocumented u64 argument. Investigate.
            "cmdline_hide" => Ok(Self::CmdlineHide),
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
    #[error("Failed to parse win_viewport event")]
    WinViewport,
    #[error("Failed to parse tabline_update event")]
    TablineUpdate,
    #[error("Failed to parse msg_showmode event")]
    MsgShowmode,
    #[error("Failed to parse msg_showcmd event")]
    MsgShowcmd,
}

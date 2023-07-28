mod default_colors_set;
mod grid_clear;
mod grid_cursor_goto;
mod grid_destroy;
mod grid_resize;
mod hl_attr_define;
mod hl_group_set;
mod mode_change;
mod mode_info_set;
mod option_set;
mod set_icon;
mod set_title;
mod util;

use self::{
    default_colors_set::{DefaultColorsSet, DefaultColorsSetParseError},
    grid_clear::{GridClear, GridClearParseError},
    grid_cursor_goto::{GridCursorGoto, GridCursorGotoParseError},
    grid_destroy::{GridDestroy, GridDestroyParseError},
    grid_resize::{GridResize, GridResizeParseError},
    hl_attr_define::{HlAttrDefine, HlAttrDefineParseError},
    hl_group_set::{HlGroupSet, HlGroupSetParseError},
    mode_change::{ModeChange, ModeChangeParseError},
    mode_info_set::{ModeInfoSet, ModeInfoSetParseError},
    option_set::{OptionSet, OptionSetParseError},
    set_icon::{SetIcon, SetIconParseError},
    set_title::{SetTitle, SetTitleParseError},
    util::parse_array,
};
use crate::event::util::parse_string;
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    SetTitle(SetTitle),
    SetIcon(SetIcon),
    OptionSet(OptionSet),
    GridClear(GridClear),
    GridDestroy(GridDestroy),
    DefaultColorsSet(DefaultColorsSet),
    HlAttrDefine(HlAttrDefine),
    ModeChange(ModeChange),
    ModeInfoSet(ModeInfoSet),
    HlGroupSet(HlGroupSet),
    GridCursorGoto(GridCursorGoto),
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

event_from!(GridResize);
event_from!(SetTitle);
event_from!(SetIcon);
event_from!(OptionSet);
event_from!(GridClear);
event_from!(GridDestroy);
event_from!(DefaultColorsSet);
event_from!(HlAttrDefine);
event_from!(ModeChange);
event_from!(ModeInfoSet);
event_from!(HlGroupSet);
event_from!(GridCursorGoto);

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let array = parse_array(value).ok_or(EventParseError::Malformed)?;
        let mut iter = array.into_iter();
        let event_name = iter.next().ok_or(EventParseError::Malformed)?;
        let mut next = || iter.next().ok_or(EventParseError::Malformed);
        let event_name = parse_string(event_name).ok_or(EventParseError::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => Ok(GridResize::try_from(next()?)?.into()),
            "set_title" => Ok(SetTitle::try_from(next()?)?.into()),
            "set_icon" => Ok(SetIcon::try_from(next()?)?.into()),
            "option_set" => Ok(OptionSet::try_from(iter)?.into()),
            "grid_clear" => Ok(GridClear::try_from(next()?)?.into()),
            "grid_destroy" => Ok(GridDestroy::try_from(next()?)?.into()),
            "default_colors_set" => Ok(DefaultColorsSet::try_from(next()?)?.into()),
            "hl_attr_define" => Ok(HlAttrDefine::try_from(iter)?.into()),
            "mode_change" => Ok(ModeChange::try_from(next()?)?.into()),
            "mode_info_set" => Ok(ModeInfoSet::try_from(next()?)?.into()),
            "hl_group_set" => Ok(HlGroupSet::try_from(iter)?.into()),
            "grid_cursor_goto" => Ok(GridCursorGoto::try_from(next()?)?.into()),
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
    #[error("{0}")]
    GridResize(#[from] GridResizeParseError),
    #[error("{0}")]
    SetTitle(#[from] SetTitleParseError),
    #[error("{0}")]
    SetIcon(#[from] SetIconParseError),
    #[error("{0}")]
    OptionSet(#[from] OptionSetParseError),
    #[error("{0}")]
    GridClear(#[from] GridClearParseError),
    #[error("{0}")]
    GridDestroy(#[from] GridDestroyParseError),
    #[error("{0}")]
    DefaultColorsSet(#[from] DefaultColorsSetParseError),
    #[error("{0}")]
    HlAttrDefine(#[from] HlAttrDefineParseError),
    #[error("{0}")]
    ModeChange(#[from] ModeChangeParseError),
    #[error("{0}")]
    ModeInfoSet(#[from] ModeInfoSetParseError),
    #[error("{0}")]
    HlGroupSet(#[from] HlGroupSetParseError),
    #[error("{0}")]
    GridCursorGoto(#[from] GridCursorGotoParseError),
}

mod cmdline_show;
mod default_colors_set;
mod grid_cursor_goto;
mod grid_line;
mod grid_resize;
mod grid_scroll;
mod hl_attr_define;
mod hl_group_set;
mod message_content;
mod messagepack_ext_types;
mod mode_change;
mod mode_info_set;
mod option_set;
mod tabline_update;
mod util;
mod win_float_pos;
mod win_pos;
mod win_viewport;

use self::{
    cmdline_show::CmdlineShow, default_colors_set::DefaultColorsSet,
    grid_cursor_goto::GridCursorGoto, grid_line::GridLine, grid_resize::GridResize,
    grid_scroll::GridScroll, hl_attr_define::HlAttrDefine, hl_group_set::HlGroupSet,
    message_content::MessageContent, mode_change::ModeChange, mode_info_set::ModeInfoSet,
    option_set::OptionSet, tabline_update::TablineUpdate, util::Parse, util::Values,
    win_float_pos::WinFloatPos, win_pos::WinPos, win_viewport::WinViewport,
};
use nvim_rs::Value;

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
    CmdlineShow(Vec<CmdlineShow>),
    WinPos(Vec<WinPos>),
    WinFloatPos(Vec<WinFloatPos>),
    MsgRuler(Vec<MessageContent>),
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
event_from!(CmdlineShow);
event_from!(WinPos);
event_from!(WinFloatPos);

fn unique<T: Parse>(iter: Values, error: Error) -> Result<Event, Error>
where
    Vec<T>: Into<Event>,
{
    iter.map().ok_or(error).map(Into::into)
}

fn shared<T: Parse>(
    iter: Values,
    event_variant: fn(Vec<T>) -> Event,
    error: Error,
) -> Result<Event, Error> {
    Ok(event_variant(
        iter.map_with(|v| Vec::parse(v)?.into_iter().next())
            .ok_or(error)?,
    ))
}

impl TryFrom<Value> for Event {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut iter = Values::new(value).ok_or(Error::Malformed)?;
        let event_name: String = iter.next().ok_or(Error::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => unique::<GridResize>(iter, Error::GridResize),
            "set_title" => shared(iter, Self::SetTitle, Error::SetTitle),
            "set_icon" => shared(iter, Self::SetIcon, Error::SetIcon),
            "option_set" => unique::<OptionSet>(iter, Error::OptionSet),
            "grid_clear" => shared(iter, Event::GridClear, Error::GridClear),
            "grid_destroy" => shared(iter, Event::GridDestroy, Error::GridDestroy),
            "default_colors_set" => unique::<DefaultColorsSet>(iter, Error::DefaultColorsSet),
            "hl_attr_define" => unique::<HlAttrDefine>(iter, Error::HlAttrDefine),
            "mode_change" => unique::<ModeChange>(iter, Error::ModeChange),
            "mode_info_set" => unique::<ModeInfoSet>(iter, Error::ModeInfoSet),
            "hl_group_set" => unique::<HlGroupSet>(iter, Error::HlGroupSet),
            "grid_cursor_goto" => unique::<GridCursorGoto>(iter, Error::GridCursorGoto),
            "grid_scroll" => unique::<GridScroll>(iter, Error::GridScroll),
            "grid_line" => unique::<GridLine>(iter, Error::GridLine),
            "win_viewport" => unique::<WinViewport>(iter, Error::WinViewport),
            "tabline_update" => unique::<TablineUpdate>(iter, Error::TablineUpdate),
            "msg_showmode" => shared(iter, Self::MsgShowmode, Error::MsgShowmode),
            "msg_showcmd" => shared(iter, Self::MsgShowcmd, Error::MsgShowcmd),
            "cmdline_show" => unique::<CmdlineShow>(iter, Error::CmdlineShow),
            "win_pos" => unique::<WinPos>(iter, Error::WinPos),
            "win_float_pos" => unique::<WinFloatPos>(iter, Error::WinFloatPos),
            "msg_ruler" => shared(iter, Self::MsgRuler, Error::MsgRuler),
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
    #[error("Failed to parse cmdline_show event")]
    CmdlineShow,
    #[error("Failed to parse win_pos event")]
    WinPos,
    #[error("Failed to parse win_float_pos event")]
    WinFloatPos,
    #[error("Failed to parse msg_ruler event")]
    MsgRuler,
}

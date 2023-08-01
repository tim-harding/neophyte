mod cmdline_pos;
mod cmdline_show;
mod cmdline_special_char;
mod default_colors_set;
mod global_event;
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
mod msg_history_show;
mod msg_set_pos;
mod msg_show;
mod option_set;
mod popupmenu_show;
mod set_icon;
mod set_title;
mod tabline_update;
mod util;
mod win_external_position;
mod win_extmark;
mod win_float_pos;
mod win_pos;
mod win_viewport;

use self::{
    cmdline_pos::CmdlinePos, cmdline_show::CmdlineShow, cmdline_special_char::CmdlineSpecialChar,
    default_colors_set::DefaultColorsSet, global_event::GlobalEvent,
    grid_cursor_goto::GridCursorGoto, grid_line::GridLine, grid_resize::GridResize,
    grid_scroll::GridScroll, hl_attr_define::HlAttrDefine, hl_group_set::HlGroupSet,
    message_content::Content, mode_change::ModeChange, mode_info_set::ModeInfoSet,
    msg_history_show::MsgHistoryShow, msg_set_pos::MsgSetPos, msg_show::MsgShow,
    option_set::OptionSet, popupmenu_show::PopupmenuShow, set_icon::SetIcon, set_title::SetTitle,
    tabline_update::TablineUpdate, util::Parse, util::Values,
    win_external_position::WinExternalPos, win_extmark::WinExtmark, win_float_pos::WinFloatPos,
    win_pos::WinPos, win_viewport::WinViewport,
};
use nvim_rs::Value;

/// A UI event sent by the Neovim instance. See here for detailed documentation:
/// https://neovim.io/doc/user/ui.html
#[derive(Debug, Clone)]
pub enum Event {
    MsgHistoryShow(MsgHistoryShow),
    CmdlineSpecialChar(CmdlineSpecialChar),
    PopupmenuShow(PopupmenuShow),
    CmdlinePos(CmdlinePos),
    GridResize(GridResize),
    SetTitle(SetTitle),
    SetIcon(SetIcon),
    OptionSet(OptionSet),
    GridClear(u64),
    GridDestroy(u64),
    DefaultColorsSet(DefaultColorsSet),
    HlAttrDefine(HlAttrDefine),
    ModeChange(ModeChange),
    ModeInfoSet(ModeInfoSet),
    HlGroupSet(HlGroupSet),
    GridCursorGoto(GridCursorGoto),
    GridScroll(GridScroll),
    GridLine(GridLine),
    WinViewport(WinViewport),
    TablineUpdate(TablineUpdate),
    MsgShowmode(Content),
    MsgShowcmd(Content),
    CmdlineShow(CmdlineShow),
    WinPos(WinPos),
    WinFloatPos(WinFloatPos),
    MsgRuler(Content),
    WinHide(u64),
    WinClose(u64),
    WinExternalPos(WinExternalPos),
    MsgSetPos(MsgSetPos),
    MsgShow(MsgShow),
    WinExtmark(WinExtmark),
    PopupmenuSelect(i64),
    CmdlineBlockShow(Vec<Content>),
    CmdlineBlockAppend(Content),
    GlobalEvent(GlobalEvent),
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

event_from!(MsgHistoryShow);
event_from!(CmdlineSpecialChar);
event_from!(PopupmenuShow);
event_from!(CmdlinePos);
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
event_from!(WinExternalPos);
event_from!(MsgSetPos);
event_from!(MsgShow);
event_from!(WinExtmark);
event_from!(SetTitle);
event_from!(SetIcon);

impl From<GlobalEvent> for Event {
    fn from(value: GlobalEvent) -> Self {
        Self::GlobalEvent(value)
    }
}

fn unique<T: Parse>(iter: Values, error: Error) -> Result<Vec<Event>, Error>
where
    T: Into<Event>,
{
    let v: Option<Vec<_>> = iter
        .into_inner()
        .map(|v| Some(T::parse(v)?.into()))
        .collect();
    v.ok_or(error)
}

fn shared<T: Parse>(
    iter: Values,
    event_variant: fn(T) -> Event,
    error: Error,
) -> Result<Vec<Event>, Error> {
    let v: Option<Vec<_>> = iter
        .into_inner()
        .map(|v| Some(event_variant(Vec::parse(v)?.into_iter().next()?)))
        .collect();
    v.ok_or(error)
}

impl Event {
    pub fn try_parse(value: Value) -> Result<Vec<Self>, Error> {
        let mut iter = Values::new(value).ok_or(Error::Malformed)?;
        let event_name: String = iter.next().ok_or(Error::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => unique::<GridResize>(iter, Error::GridResize),
            "set_title" => unique::<SetTitle>(iter, Error::SetTitle),
            "set_icon" => unique::<SetIcon>(iter, Error::SetIcon),
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
            "win_hide" => shared(iter, Self::WinHide, Error::WinHide),
            "win_close" => shared(iter, Self::WinClose, Error::WinClose),
            "win_external_pos" => unique::<WinExternalPos>(iter, Error::WinExternalPos),
            "msg_set_pos" => unique::<MsgSetPos>(iter, Error::MsgSetPos),
            "msg_show" => unique::<MsgShow>(iter, Error::MsgShow),
            "win_extmark" => unique::<WinExtmark>(iter, Error::WinExtmark),
            "cmdline_pos" => unique::<CmdlinePos>(iter, Error::CmdlinePos),
            "popupmenu_show" => unique::<PopupmenuShow>(iter, Error::PopupmenuShow),
            "cmdline_special_char" => unique::<CmdlineSpecialChar>(iter, Error::CmdlineSpecialChar),
            "msg_history_show" => unique::<MsgHistoryShow>(iter, Error::MsgHistoryShow),
            "popupmenu_select" => shared(iter, Self::PopupmenuSelect, Error::PopupmenuSelect),
            "cmdline_block_show" => shared(iter, Self::CmdlineBlockShow, Error::CmdlineBlockShow),
            "cmdline_block_append" => {
                shared(iter, Self::CmdlineBlockAppend, Error::CmdlineBlockAppend)
            }
            _ => Ok(vec![GlobalEvent::try_from(event_name.as_str())
                .map(Into::into)
                .map_err(|_| Error::UnknownEvent(event_name))?]),
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
    #[error("Failed to parse win_hide event")]
    WinHide,
    #[error("Failed to parse win_close event")]
    WinClose,
    #[error("Failed to parse win_external_pos event")]
    WinExternalPos,
    #[error("Failed to parse msg_set_pos event")]
    MsgSetPos,
    #[error("Failed to parse msg_show event")]
    MsgShow,
    #[error("Failed to parse win_extmark event")]
    WinExtmark,
    #[error("Failed to parse cmdline_pos event")]
    CmdlinePos,
    #[error("Failed to parse popupmenu_select event")]
    PopupmenuSelect,
    #[error("Failed to parse popupmenu_show event")]
    PopupmenuShow,
    #[error("Failed to parse cmdline_special_char event")]
    CmdlineSpecialChar,
    #[error("Failed to parse cmdline_block_show event")]
    CmdlineBlockShow,
    #[error("Failed to parse cmdline_block_append event")]
    CmdlineBlockAppend,
    #[error("Failed to parse msg_history_show event")]
    MsgHistoryShow,
}

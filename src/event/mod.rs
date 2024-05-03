mod chdir;
mod cmdline_block_append;
mod cmdline_block_show;
mod cmdline_pos;
mod cmdline_show;
mod cmdline_special_char;
mod default_colors_set;
mod grid_clear;
mod grid_cursor_goto;
mod grid_destroy;
pub mod grid_line;
mod grid_resize;
mod grid_scroll;
pub mod hl_attr_define;
mod hl_group_set;
mod message_content;
mod messagepack_ext_types;
mod mode_change;
pub mod mode_info_set;
pub mod msg_history_show;
mod msg_ruler;
mod msg_set_pos;
pub mod msg_show;
mod msg_showcmd;
mod msg_showmode;
pub mod option_set;
mod popupmenu_select;
mod popupmenu_show;
pub mod rgb;
mod set_icon;
mod set_title;
mod tabline_update;
mod win_close;
mod win_external_position;
mod win_extmark;
mod win_float_pos;
mod win_hide;
mod win_pos;
mod win_viewport;

pub use self::{
    chdir::Chdir,
    cmdline_block_append::CmdlineBlockAppend,
    cmdline_block_show::CmdlineBlockShow,
    cmdline_pos::CmdlinePos,
    cmdline_show::CmdlineShow,
    cmdline_special_char::CmdlineSpecialChar,
    default_colors_set::DefaultColorsSet,
    grid_clear::GridClear,
    grid_cursor_goto::GridCursorGoto,
    grid_destroy::GridDestroy,
    grid_line::GridLine,
    grid_resize::GridResize,
    grid_scroll::GridScroll,
    hl_attr_define::HlAttrDefine,
    hl_group_set::HlGroupSet,
    message_content::Content,
    mode_change::ModeChange,
    mode_info_set::ModeInfoSet,
    msg_history_show::MsgHistoryShow,
    msg_ruler::MsgRuler,
    msg_set_pos::MsgSetPos,
    msg_show::MsgShow,
    msg_showcmd::MsgShowcmd,
    msg_showmode::MsgShowmode,
    option_set::OptionSet,
    popupmenu_select::PopupmenuSelect,
    popupmenu_show::PopupmenuShow,
    set_icon::SetIcon,
    set_title::SetTitle,
    tabline_update::TablineUpdate,
    win_close::WinClose,
    win_external_position::WinExternalPos,
    win_extmark::WinExtmark,
    win_float_pos::{Anchor, WinFloatPos},
    win_hide::WinHide,
    win_pos::WinPos,
    win_viewport::WinViewport,
};
use crate::util::{Parse, Values};
use rmpv::Value;

/// A UI event sent by the Neovim instance. See here for detailed documentation:
/// https://neovim.io/doc/user/ui.html
#[derive(Debug, Clone)]
pub enum Event {
    GridResize(GridResize),
    GridClear(GridClear),
    GridDestroy(GridDestroy),
    GridCursorGoto(GridCursorGoto),
    GridScroll(GridScroll),
    GridLine(GridLine),

    WinViewport(WinViewport),
    WinPos(WinPos),
    WinFloatPos(WinFloatPos),
    WinHide(WinHide),
    WinClose(WinClose),
    WinExternalPos(WinExternalPos),
    WinExtmark(WinExtmark),

    MsgHistoryShow(MsgHistoryShow),
    MsgShowmode(MsgShowmode),
    MsgShowcmd(MsgShowcmd),
    MsgRuler(MsgRuler),
    MsgSetPos(MsgSetPos),
    MsgShow(MsgShow),

    CmdlineShow(CmdlineShow),
    CmdlineSpecialChar(CmdlineSpecialChar),
    CmdlinePos(CmdlinePos),
    CmdlineBlockShow(CmdlineBlockShow),
    CmdlineBlockAppend(CmdlineBlockAppend),

    PopupmenuShow(PopupmenuShow),
    PopupmenuSelect(PopupmenuSelect),

    SetTitle(SetTitle),
    SetIcon(SetIcon),
    OptionSet(OptionSet),
    DefaultColorsSet(DefaultColorsSet),
    HlAttrDefine(HlAttrDefine),
    ModeChange(ModeChange),
    ModeInfoSet(ModeInfoSet),
    HlGroupSet(HlGroupSet),
    TablineUpdate(TablineUpdate),
    Chdir(Chdir),

    /// The mouse was enabled in the current editor mode
    MouseOn,
    /// The mouse was disabled in the current editor mode
    MouseOff,
    /// The UI must stop rendering the cursor
    BusyStart,
    /// The UI must resume rendering the cursor
    BusyStop,
    /// :suspend command or CTRL-Z mapping is used. A terminal client could
    /// suspend itself. Other clients can safely ignore it.
    Suspend,
    /// The menu mappings changed
    UpdateMenu,
    /// Notify the user with an audible bell
    Bell,
    /// Notify the user with a visual bell
    VisualBell,
    /// Nvim is done redrawing the screen. For an implementation that renders to
    /// an internal buffer, this is the time to display the redrawn parts to the
    /// user.
    Flush,
    /// Hide the cmdline
    CmdlineHide,
    /// Show a block of text to the current command line. Similar to to
    /// cmdline_show but allows for multiple lines
    CmdlineBlockHide,
    /// Hide the popupmenu
    PopupmenuHide,
    /// Clear all messages currently displayed by "msg_show". Messages sent by
    /// other "msg_" events below will not be affected.
    MsgClear,
    /// Clear the messages history
    MsgHistoryClear,
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
event_from!(GridClear);
event_from!(GridDestroy);
event_from!(MsgShowmode);
event_from!(MsgShowcmd);
event_from!(MsgRuler);
event_from!(WinHide);
event_from!(WinClose);
event_from!(PopupmenuSelect);
event_from!(CmdlineBlockShow);
event_from!(CmdlineBlockAppend);
event_from!(Chdir);

fn parse<T>(iter: Values, error: Error) -> Result<Vec<Event>, Error>
where
    T: Parse + Into<Event>,
{
    let v: Option<Vec<_>> = iter
        .into_inner()
        .map(|v| Some(T::parse(v)?.into()))
        .collect();
    v.ok_or(error)
}

impl Event {
    pub fn try_parse(value: Value) -> Result<Vec<Self>, Error> {
        let mut iter = Values::new(value).ok_or(Error::Malformed)?;
        let event_name: String = iter.next().ok_or(Error::Malformed)?;
        match event_name.as_str() {
            "grid_resize" => parse::<GridResize>(iter, Error::GridResize),
            "set_title" => parse::<SetTitle>(iter, Error::SetTitle),
            "set_icon" => parse::<SetIcon>(iter, Error::SetIcon),
            "option_set" => parse::<OptionSet>(iter, Error::OptionSet),
            "grid_clear" => parse::<GridClear>(iter, Error::GridClear),
            "grid_destroy" => parse::<GridDestroy>(iter, Error::GridDestroy),
            "default_colors_set" => parse::<DefaultColorsSet>(iter, Error::DefaultColorsSet),
            "hl_attr_define" => parse::<HlAttrDefine>(iter, Error::HlAttrDefine),
            "mode_change" => parse::<ModeChange>(iter, Error::ModeChange),
            "mode_info_set" => parse::<ModeInfoSet>(iter, Error::ModeInfoSet),
            "hl_group_set" => parse::<HlGroupSet>(iter, Error::HlGroupSet),
            "grid_cursor_goto" => parse::<GridCursorGoto>(iter, Error::GridCursorGoto),
            "grid_scroll" => parse::<GridScroll>(iter, Error::GridScroll),
            "grid_line" => parse::<GridLine>(iter, Error::GridLine),
            "win_viewport" => parse::<WinViewport>(iter, Error::WinViewport),
            "tabline_update" => parse::<TablineUpdate>(iter, Error::TablineUpdate),
            "msg_showmode" => parse::<MsgShowmode>(iter, Error::MsgShowmode),
            "msg_showcmd" => parse::<MsgShowcmd>(iter, Error::MsgShowcmd),
            "cmdline_show" => parse::<CmdlineShow>(iter, Error::CmdlineShow),
            "win_pos" => parse::<WinPos>(iter, Error::WinPos),
            "win_float_pos" => parse::<WinFloatPos>(iter, Error::WinFloatPos),
            "msg_ruler" => parse::<MsgRuler>(iter, Error::MsgRuler),
            "win_hide" => parse::<WinHide>(iter, Error::WinHide),
            "win_close" => parse::<WinClose>(iter, Error::WinClose),
            "win_external_pos" => parse::<WinExternalPos>(iter, Error::WinExternalPos),
            "msg_set_pos" => parse::<MsgSetPos>(iter, Error::MsgSetPos),
            "msg_show" => parse::<MsgShow>(iter, Error::MsgShow),
            "win_extmark" => parse::<WinExtmark>(iter, Error::WinExtmark),
            "cmdline_pos" => parse::<CmdlinePos>(iter, Error::CmdlinePos),
            "popupmenu_show" => parse::<PopupmenuShow>(iter, Error::PopupmenuShow),
            "cmdline_special_char" => parse::<CmdlineSpecialChar>(iter, Error::CmdlineSpecialChar),
            "msg_history_show" => parse::<MsgHistoryShow>(iter, Error::MsgHistoryShow),
            "popupmenu_select" => parse::<PopupmenuSelect>(iter, Error::PopupmenuSelect),
            "cmdline_block_show" => parse::<CmdlineBlockShow>(iter, Error::CmdlineBlockShow),
            "cmdline_block_append" => parse::<CmdlineBlockAppend>(iter, Error::CmdlineBlockAppend),
            "chdir" => parse::<Chdir>(iter, Error::Chdir),
            "mouse_on" => Ok(vec![Self::MouseOn]),
            "mouse_off" => Ok(vec![Self::MouseOff]),
            "busy_start" => Ok(vec![Self::BusyStart]),
            "busy_stop" => Ok(vec![Self::BusyStop]),
            "suspend" => Ok(vec![Self::Suspend]),
            "update_menu" => Ok(vec![Self::UpdateMenu]),
            "bell" => Ok(vec![Self::Bell]),
            "visual_bell" => Ok(vec![Self::VisualBell]),
            "flush" => Ok(vec![Self::Flush]),
            "cmdline_hide" => Ok(vec![Self::CmdlineHide]),
            "cmdline_block_hide" => Ok(vec![Self::CmdlineBlockHide]),
            "popupmenu_hide" => Ok(vec![Self::PopupmenuHide]),
            "msg_clear" => Ok(vec![Self::MsgClear]),
            "msg_history_clear" => Ok(vec![Self::MsgHistoryClear]),
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
    #[error("Failed to parse chdir event")]
    Chdir,
    #[error("Failed to parse msg_history_show event")]
    MsgHistoryShow,
}

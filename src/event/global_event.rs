/// An event without parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlobalEvent {
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

impl TryFrom<String> for GlobalEvent {
    type Error = GlobalEventUnknown;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "mouse_on" => Self::MouseOn,
            "mouse_off" => Self::MouseOff,
            "busy_start" => Self::BusyStart,
            "busy_stop" => Self::BusyStop,
            "suspend" => Self::Suspend,
            "update_menu" => Self::UpdateMenu,
            "bell" => Self::Bell,
            "visual_bell" => Self::VisualBell,
            "flush" => Self::Flush,
            "cmdline_hide" => Self::CmdlineHide,
            "cmdline_block_hide" => Self::CmdlineBlockHide,
            "popupmenu_hide" => Self::PopupmenuHide,
            "msg_clear" => Self::MsgClear,
            "msg_history_clear" => Self::MsgHistoryClear,
            _ => return Err(GlobalEventUnknown),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("Unrecognized event name")]
pub struct GlobalEventUnknown;

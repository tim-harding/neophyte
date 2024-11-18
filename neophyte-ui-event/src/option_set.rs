use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// UI-related option change.
///
/// Options are not represented here if their effects are communicated in other
/// UI events. For example, instead of forwarding the 'mouse' option value, the
/// "mouse_on" and "mouse_off" UI events directly indicate if mouse support is
/// active. Some options like 'ambiwidth' have already taken effect on the grid,
/// where appropriate empty cells are added, however a UI might still use such
/// options when rendering raw text sent from Nvim, like for ui-cmdline.
#[derive(Debug, Clone, Serialize)]
pub enum OptionSet {
    /// See https://neovim.io/doc/user/options.html#'arabicshape'
    Arabicshape(bool),
    /// See https://neovim.io/doc/user/options.html#'ambiwidth'
    Ambiwidth(Ambiwidth),
    /// See https://neovim.io/doc/user/options.html#'emoji'
    Emoji(bool),
    /// See https://neovim.io/doc/user/options.html#'guifont'
    Guifont(String),
    /// See https://neovim.io/doc/user/options.html#'guifontwide'
    Guifontwide(String),
    /// See https://neovim.io/doc/user/options.html#'linespace'
    Linespace(u32),
    /// See https://neovim.io/doc/user/options.html#'mousefocus'
    Mousefocus(bool),
    /// See https://neovim.io/doc/user/options.html#'mousemoveevent'
    Mousemoveevent(bool),
    /// See https://neovim.io/doc/user/options.html#'pumblend'
    Pumblend(u32),
    /// See https://neovim.io/doc/user/options.html#'showtabline'
    Showtabline(Showtabline),
    /// See https://neovim.io/doc/user/options.html#'termguicolors'
    Termguicolors(bool),
    /// Externalize the cmdline
    ExtCmdline(bool),
    /// Detailed highlight state
    ExtHlstate(bool),
    /// Line-based grid events
    ExtLinegrid(bool),
    /// Externalize messages
    ExtMessages(bool),
    /// Per-window grid events
    ExtMultigrid(bool),
    /// Externalize popupmenu completion
    ExtPopupmenu(bool),
    /// Externalize the tabline
    ExtTabline(bool),
    /// Use external default colors
    ExtTermcolors(bool),
    /// Sets the name of the default terminal type
    TermName(String),
    /// Sets the number of supported colors t_Co
    TermColors(u32),
    /// Sets the default value of background
    TermBackground(u32),
    /// Read buffer 1 from this fd as if it were stdin --. Only from --embed UI on startup.
    StdinFd(u32),
    /// Tells if stdin is a TTY
    StdinTty(bool),
    /// Tells if stdout is a TTY
    StdoutTty(bool),
    /// An option not enumerated in the option_set documentation
    Other { name: String, value: Value },
}

impl Parse for OptionSet {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        let name: String = iter.next()?;
        Some(match name.as_str() {
            "arabicshape" => Self::Arabicshape(iter.next()?),
            "ambiwidth" => Self::Ambiwidth(iter.next()?),
            "emoji" => Self::Emoji(iter.next()?),
            "guifont" => Self::Guifont(iter.next()?),
            "guifontwide" => Self::Guifontwide(iter.next()?),
            "linespace" => Self::Linespace(iter.next()?),
            "mousefocus" => Self::Mousefocus(iter.next()?),
            "mousemoveevent" => Self::Mousemoveevent(iter.next()?),
            "pumblend" => Self::Pumblend(iter.next()?),
            "showtabline" => Self::Showtabline(iter.next()?),
            "termguicolors" => Self::Termguicolors(iter.next()?),
            "ext_cmdline" => Self::ExtCmdline(iter.next()?),
            "ext_hlstate" => Self::ExtHlstate(iter.next()?),
            "ext_linegrid" => Self::ExtLinegrid(iter.next()?),
            "ext_messages" => Self::ExtMessages(iter.next()?),
            "ext_multigrid" => Self::ExtMultigrid(iter.next()?),
            "ext_popupmenu" => Self::ExtPopupmenu(iter.next()?),
            "ext_tabline" => Self::ExtTabline(iter.next()?),
            "ext_termcolors" => Self::ExtTermcolors(iter.next()?),
            "term_name" => Self::TermName(iter.next()?),
            "term_colors" => Self::TermColors(iter.next()?),
            "term_background" => Self::TermBackground(iter.next()?),
            "stdin_fd" => Self::StdinFd(iter.next()?),
            "stdin_tty" => Self::StdinTty(iter.next()?),
            "stdout_tty" => Self::StdoutTty(iter.next()?),
            _ => Self::Other {
                name,
                value: iter.next()?,
            },
        })
    }
}

#[derive(Debug, Clone, Default, Serialize)]
/// Tells Vim what to do with characters with East Asian Width Class Ambiguous
pub enum Ambiwidth {
    /// Use the same width as characters in US-ASCII
    #[default]
    Single,
    /// Use twice the width of ASCII characters
    Double,
}

impl Parse for Ambiwidth {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        match s.as_str() {
            "single" => Some(Self::Single),
            "double" => Some(Self::Double),
            _ => None,
        }
    }
}

/// When the line with tab page labels will be displayed
#[derive(Debug, Clone, Default, Serialize)]
pub enum Showtabline {
    #[default]
    Never,
    /// Only if there are at least two tab pages
    Sometimes,
    Always,
}

impl Parse for Showtabline {
    fn parse(value: Value) -> Option<Self> {
        Some(match u32::parse(value)? {
            0 => Self::Never,
            1 => Self::Sometimes,
            2 => Self::Always,
            _ => return None,
        })
    }
}

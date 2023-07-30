use super::util::{parse_array, parse_bool, parse_string, parse_u64};
use nvim_rs::Value;

/// UI-related option change.
///
/// Options are not represented here if their effects are communicated in other
/// UI events. For example, instead of forwarding the 'mouse' option value, the
/// "mouse_on" and "mouse_off" UI events directly indicate if mouse support is
/// active. Some options like 'ambiwidth' have already taken effect on the grid,
/// where appropriate empty cells are added, however a UI might still use such
/// options when rendering raw text sent from Nvim, like for ui-cmdline.
#[derive(Debug, Clone)]
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
    Linespace(u64),
    /// See https://neovim.io/doc/user/options.html#'mousefocus'
    Mousefocus(bool),
    /// See https://neovim.io/doc/user/options.html#'mousemoveevent'
    Mousemoveevent(bool),
    /// See https://neovim.io/doc/user/options.html#'pumblend'
    Pumblend(u64),
    /// See https://neovim.io/doc/user/options.html#'showtabline'
    Showtabline(u64),
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
    TermColors(u64),
    /// Sets the default value of background
    TermBackground(u64),
    /// Read buffer 1 from this fd as if it were stdin --. Only from --embed UI on startup.
    StdinFd(u64),
    /// Tells if stdin is a TTY
    StdinTty(bool),
    /// Tells if stdout is a TTY
    StdoutTty(bool),
    /// An option not enumerated in the option_set documentation
    Other { name: String, value: Value },
}

impl OptionSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        let name = parse_string(iter.next()?)?;
        let value = iter.next()?;
        Some(match name.as_str() {
            "arabicshape" => Self::Arabicshape(parse_bool(value)?),
            "ambiwidth" => Self::Ambiwidth(Ambiwidth::parse(value)?),
            "emoji" => Self::Emoji(parse_bool(value)?),
            "guifont" => Self::Guifont(parse_string(value)?),
            "guifontwide" => Self::Guifontwide(parse_string(value)?),
            "linespace" => Self::Linespace(parse_u64(value)?),
            "mousefocus" => Self::Mousefocus(parse_bool(value)?),
            "mousemoveevent" => Self::Mousemoveevent(parse_bool(value)?),
            "pumblend" => Self::Pumblend(parse_u64(value)?),
            "showtabline" => Self::Showtabline(parse_u64(value)?),
            "termguicolors" => Self::Termguicolors(parse_bool(value)?),
            "ext_cmdline" => Self::ExtCmdline(parse_bool(value)?),
            "ext_hlstate" => Self::ExtHlstate(parse_bool(value)?),
            "ext_linegrid" => Self::ExtLinegrid(parse_bool(value)?),
            "ext_messages" => Self::ExtMessages(parse_bool(value)?),
            "ext_multigrid" => Self::ExtMultigrid(parse_bool(value)?),
            "ext_popupmenu" => Self::ExtPopupmenu(parse_bool(value)?),
            "ext_tabline" => Self::ExtTabline(parse_bool(value)?),
            "ext_termcolors" => Self::ExtTermcolors(parse_bool(value)?),
            "term_name" => Self::TermName(parse_string(value)?),
            "term_colors" => Self::TermColors(parse_u64(value)?),
            "term_background" => Self::TermBackground(parse_u64(value)?),
            "stdin_fd" => Self::StdinFd(parse_u64(value)?),
            "stdin_tty" => Self::StdinTty(parse_bool(value)?),
            "stdout_tty" => Self::StdoutTty(parse_bool(value)?),
            _ => Self::Other { name, value },
        })
    }
}

#[derive(Debug, Clone)]
/// Tells Vim what to do with characters with East Asian Width Class Ambiguous
pub enum Ambiwidth {
    /// Use the same width as characters in US-ASCII
    Single,
    /// Use twice the width of ASCII characters
    Double,
}

impl Ambiwidth {
    fn parse(value: Value) -> Option<Self> {
        let s = parse_string(value)?;
        match s.as_str() {
            "single" => Some(Self::Single),
            "double" => Some(Self::Double),
            _ => None,
        }
    }
}

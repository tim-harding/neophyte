use super::util::{parse_array, parse_bool, parse_string, parse_u64};
use nvim_rs::Value;

/// UI-related option change
#[derive(Debug, Clone)]
pub enum OptionSet {
    Arabicshape(bool),
    Ambiwidth(String),
    Emoji(bool),
    Guifont(String),
    Guifontwide(String),
    Linespace(u64),
    Mousefocus(bool),
    Mousemoveevent(bool),
    Pumblend(u64),
    Showtabline(u64),
    Termguicolors(bool),
    ExtCmdline(bool),
    ExtHlstate(bool),
    ExtLinegrid(bool),
    ExtMessages(bool),
    ExtMultigrid(bool),
    ExtPopupmenu(bool),
    ExtTabline(bool),
    ExtTermcolors(bool),
    TermName(String),
    TermColors(u64),
    TermBackground(u64),
    StdinFd(u64),
    StdinTty(bool),
    StdoutTty(bool),
}

impl OptionSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        let option_name = parse_string(iter.next()?)?;
        let option_value = iter.next()?;
        Some(match option_name.as_str() {
            "arabicshape" => Self::Arabicshape(parse_bool(option_value)?),
            "ambiwidth" => Self::Ambiwidth(parse_string(option_value)?),
            "emoji" => Self::Emoji(parse_bool(option_value)?),
            "guifont" => Self::Guifont(parse_string(option_value)?),
            "guifontwide" => Self::Guifontwide(parse_string(option_value)?),
            "linespace" => Self::Linespace(parse_u64(option_value)?),
            "mousefocus" => Self::Mousefocus(parse_bool(option_value)?),
            "mousemoveevent" => Self::Mousemoveevent(parse_bool(option_value)?),
            "pumblend" => Self::Pumblend(parse_u64(option_value)?),
            "showtabline" => Self::Showtabline(parse_u64(option_value)?),
            "termguicolors" => Self::Termguicolors(parse_bool(option_value)?),
            "ext_cmdline" => Self::ExtCmdline(parse_bool(option_value)?),
            "ext_hlstate" => Self::ExtHlstate(parse_bool(option_value)?),
            "ext_linegrid" => Self::ExtLinegrid(parse_bool(option_value)?),
            "ext_messages" => Self::ExtMessages(parse_bool(option_value)?),
            "ext_multigrid" => Self::ExtMultigrid(parse_bool(option_value)?),
            "ext_popupmenu" => Self::ExtPopupmenu(parse_bool(option_value)?),
            "ext_tabline" => Self::ExtTabline(parse_bool(option_value)?),
            "ext_termcolors" => Self::ExtTermcolors(parse_bool(option_value)?),
            "term_name" => Self::TermName(parse_string(option_value)?),
            "term_colors" => Self::TermColors(parse_u64(option_value)?),
            "term_background" => Self::TermBackground(parse_u64(option_value)?),
            "stdin_fd" => Self::StdinFd(parse_u64(option_value)?),
            "stdin_tty" => Self::StdinTty(parse_bool(option_value)?),
            "stdout_tty" => Self::StdoutTty(parse_bool(option_value)?),
            _ => {
                eprintln!("Unknown option_set option: {option_name}");
                return None;
            }
        })
    }
}

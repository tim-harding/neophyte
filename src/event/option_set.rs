use super::util::{parse_array, parse_bool, parse_string, parse_u64};
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone, Default)]
pub struct OptionSet {
    arabicshape: Option<bool>,
    ambiwidth: Option<String>,
    emoji: Option<bool>,
    guifont: Option<String>,
    guifontwide: Option<String>,
    linespace: Option<u64>,
    mousefocus: Option<bool>,
    mousemoveevent: Option<bool>,
    pumblend: Option<u64>,
    showtabline: Option<u64>,
    termguicolors: Option<bool>,
    ext_cmdline: Option<bool>,
    ext_hlstate: Option<bool>,
    ext_linegrid: Option<bool>,
    ext_messages: Option<bool>,
    ext_multigrid: Option<bool>,
    ext_popupmenu: Option<bool>,
    ext_tabline: Option<bool>,
    ext_termcolors: Option<bool>,
    term_name: Option<String>,
    term_colors: Option<u64>,
    term_background: Option<u64>,
    stdin_fd: Option<u64>,
    stdin_tty: Option<bool>,
    stdout_tty: Option<bool>,
}

impl TryFrom<IntoIter<Value>> for OptionSet {
    type Error = OptionSetParseError;

    fn try_from(values: IntoIter<Value>) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut out = Self::default();
            for value in values {
                let array = parse_array(value)?;
                let mut iter = array.into_iter();
                let option_name = parse_string(iter.next()?)?;
                let option_value = iter.next()?;
                match option_name.as_str() {
                    "arabicshape" => out.arabicshape = Some(parse_bool(option_value)?),
                    "ambiwidth" => out.ambiwidth = Some(parse_string(option_value)?),
                    "emoji" => out.emoji = Some(parse_bool(option_value)?),
                    "guifont" => out.guifont = Some(parse_string(option_value)?),
                    "guifontwide" => out.guifontwide = Some(parse_string(option_value)?),
                    "linespace" => out.linespace = Some(parse_u64(option_value)?),
                    "mousefocus" => out.mousefocus = Some(parse_bool(option_value)?),
                    "mousemoveevent" => out.mousemoveevent = Some(parse_bool(option_value)?),
                    "pumblend" => out.pumblend = Some(parse_u64(option_value)?),
                    "showtabline" => out.showtabline = Some(parse_u64(option_value)?),
                    "termguicolors" => out.termguicolors = Some(parse_bool(option_value)?),
                    "ext_cmdline" => out.ext_cmdline = Some(parse_bool(option_value)?),
                    "ext_hlstate" => out.ext_hlstate = Some(parse_bool(option_value)?),
                    "ext_linegrid" => out.ext_linegrid = Some(parse_bool(option_value)?),
                    "ext_messages" => out.ext_messages = Some(parse_bool(option_value)?),
                    "ext_multigrid" => out.ext_multigrid = Some(parse_bool(option_value)?),
                    "ext_popupmenu" => out.ext_popupmenu = Some(parse_bool(option_value)?),
                    "ext_tabline" => out.ext_tabline = Some(parse_bool(option_value)?),
                    "ext_termcolors" => out.ext_termcolors = Some(parse_bool(option_value)?),
                    "term_name" => out.term_name = Some(parse_string(option_value)?),
                    "term_colors" => out.term_colors = Some(parse_u64(option_value)?),
                    "term_background" => out.term_background = Some(parse_u64(option_value)?),
                    "stdin_fd" => out.stdin_fd = Some(parse_u64(option_value)?),
                    "stdin_tty" => out.stdin_tty = Some(parse_bool(option_value)?),
                    "stdout_tty" => out.stdout_tty = Some(parse_bool(option_value)?),
                    _ => eprintln!("Unknown option: {option_name}"),
                }
            }
            Some(out)
        };
        inner().ok_or(OptionSetParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse option_set event")]
pub struct OptionSetParseError;

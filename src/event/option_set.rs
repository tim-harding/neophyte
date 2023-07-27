use super::util::{parse_bool, parse_string, parse_u64};
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
        let mut out = Self::default();
        for value in values {
            match value {
                Value::Array(array) => {
                    let mut iter = array.into_iter();
                    let name = parse_string(iter.next().ok_or(OptionSetParseError)?)
                        .ok_or(OptionSetParseError)?;
                    let value = iter.next().ok_or(OptionSetParseError)?;
                    match name.as_str() {
                        "arabicshape" => {
                            out.arabicshape = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ambiwidth" => {
                            out.ambiwidth = Some(parse_string(value).ok_or(OptionSetParseError)?)
                        }
                        "emoji" => out.emoji = Some(parse_bool(value).ok_or(OptionSetParseError)?),
                        "guifont" => {
                            out.guifont = Some(parse_string(value).ok_or(OptionSetParseError)?)
                        }
                        "guifontwide" => {
                            out.guifontwide = Some(parse_string(value).ok_or(OptionSetParseError)?)
                        }
                        "linespace" => {
                            out.linespace = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "mousefocus" => {
                            out.mousefocus = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "mousemoveevent" => {
                            out.mousemoveevent = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "pumblend" => {
                            out.pumblend = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "showtabline" => {
                            out.showtabline = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "termguicolors" => {
                            out.termguicolors = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_cmdline" => {
                            out.ext_cmdline = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_hlstate" => {
                            out.ext_hlstate = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_linegrid" => {
                            out.ext_linegrid = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_messages" => {
                            out.ext_messages = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_multigrid" => {
                            out.ext_multigrid = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_popupmenu" => {
                            out.ext_popupmenu = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_tabline" => {
                            out.ext_tabline = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "ext_termcolors" => {
                            out.ext_termcolors = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "term_name" => {
                            out.term_name = Some(parse_string(value).ok_or(OptionSetParseError)?)
                        }
                        "term_colors" => {
                            out.term_colors = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "term_background" => {
                            out.term_background = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "stdin_fd" => {
                            out.stdin_fd = Some(parse_u64(value).ok_or(OptionSetParseError)?)
                        }
                        "stdin_tty" => {
                            out.stdin_tty = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        "stdout_tty" => {
                            out.stdout_tty = Some(parse_bool(value).ok_or(OptionSetParseError)?)
                        }
                        _ => eprintln!("Unknown option: {name}"),
                    }
                }
                _ => return Err(OptionSetParseError),
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse option_set event")]
pub struct OptionSetParseError;

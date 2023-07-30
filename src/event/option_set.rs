use super::util::{maybe_field, parse_array, parse_bool, parse_string, parse_u64};
use nvim_rs::Value;
use std::fmt::Debug;

// TODO: Refactor as an enum

/// UI-related option change
#[derive(Clone, Default)]
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

impl OptionSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut out = Self::default();
        let mut iter = parse_array(value)?.into_iter();
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
        Some(out)
    }
}

impl Debug for OptionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("OptionSet");
        maybe_field(&mut s, "arabicshape", self.arabicshape);
        maybe_field(&mut s, "ambiwidth", self.ambiwidth.as_ref());
        maybe_field(&mut s, "emoji", self.emoji);
        maybe_field(&mut s, "guifont", self.guifont.as_ref());
        maybe_field(&mut s, "guifontwide", self.guifontwide.as_ref());
        maybe_field(&mut s, "linespace", self.linespace);
        maybe_field(&mut s, "mousefocus", self.mousefocus);
        maybe_field(&mut s, "mousemoveevent", self.mousemoveevent);
        maybe_field(&mut s, "pumblend", self.pumblend);
        maybe_field(&mut s, "showtabline", self.showtabline);
        maybe_field(&mut s, "termguicolors", self.termguicolors);
        maybe_field(&mut s, "ext_cmdline", self.ext_cmdline);
        maybe_field(&mut s, "ext_hlstate", self.ext_hlstate);
        maybe_field(&mut s, "ext_linegrid", self.ext_linegrid);
        maybe_field(&mut s, "ext_messages", self.ext_messages);
        maybe_field(&mut s, "ext_multigrid", self.ext_multigrid);
        maybe_field(&mut s, "ext_popupmenu", self.ext_popupmenu);
        maybe_field(&mut s, "ext_tabline", self.ext_tabline);
        maybe_field(&mut s, "ext_termcolors", self.ext_termcolors);
        maybe_field(&mut s, "term_name", self.term_name.as_ref());
        maybe_field(&mut s, "term_colors", self.term_colors);
        maybe_field(&mut s, "term_background", self.term_background);
        maybe_field(&mut s, "stdin_fd", self.stdin_fd);
        maybe_field(&mut s, "stdin_tty", self.stdin_tty);
        maybe_field(&mut s, "stdout_tty", self.stdout_tty);
        s.finish()
    }
}

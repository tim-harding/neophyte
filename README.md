# Neophyte

![Crates.io](https://img.shields.io/crates/d/neophyte)

Neophyte is a Neovim GUI rendered with WebGPU. It offers several niceties over
the default terminal-based UI:

- Text shaping and rasterization by [Swash](https://github.com/dfrg/swash),
  providing high-quality font rendering features such as ligatures, fallback
  fonts, and emoji
- Smooth scrolling
- Cursor animations
- Unnecessarily low input latency

This project is complete enough to use as a primary editor. For a more mature
option, try [Neovide](https://github.com/neovide/neovide).

## Configuration

### Fonts

By default, Neophyte will use the first monospaced font it finds on the system.
This will likely be quite ugly. Custom fonts can be specified with the
[guifont](https://neovim.io/doc/user/options.html#'guifont') option. Height and
width specifiers are supported, though they can only be applied globally, not to
individual fonts. The width option may give better results with respect to
overlap between Powerline symbols. For example, here is Lua code to use Cascadia
Code at eleven pixels wide with nerd font and emoji fallbacks:

```lua 
vim.opt.guifont = "Cascadia Code PL:w11, Symbols Nerd Font, Noto Color Emoji"
```

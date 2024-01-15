# Neophyte

[![Crates.io Version](https://img.shields.io/crates/v/neophyte)](https://crates.io/crates/neophyte)

Neophyte is a Neovim GUI rendered with WebGPU and written in Rust. 
It offers several niceties over the default terminal-based UI:

- Text shaping and rasterization by [Swash](https://github.com/dfrg/swash),
  providing high-quality font rendering features such as ligatures, fallback
  fonts, and emoji
- Smooth scrolling
- Cursor animations
- Pixel-level window positioning

https://github.com/tim-harding/neophyte/assets/13814470/7007c562-efaf-4e0e-98a3-cc71954183d0

## Installation

Neophyte currently works best on MacOS and Linux. It also requires a compatible
graphics driver, namely Vulkan, Metal, or DX12. Installing from Crates.io or Git
requires the [Rust toolchain](https://www.rust-lang.org/tools/install).

### Crates.io
```bash
cargo install neophyte
```

### Git
```bash
git clone https://github.com/tim-harding/neophyte
cd neophyte
git lfs install
git lfs pull
cargo build --release
```

The binary will be `target/release/neophyte`. 

### Releases
Prebuilt binaries are available in the
[releases](https://github.com/tim-harding/neophyte/releases/latest). 

## Configuration

### Scripting

Neophyte is scriptable with Lua. The API is LuaLS type-annotated for
discoverability. 

```lua
-- lazy.nvim example:
{
  'tim-harding/neophyte',
  tag = '0.2.2',
  event = 'VeryLazy',
  opts = {
    -- Same as neophyte.setup({ ... })
  },
}

-- API usage example:
local neophyte = require('neophyte')
neophyte.setup({
  fonts = {
    {
      name = 'Cascadia Code PL',
      features = {
        {
          name = 'calt',
          value = 1,
        },
        -- Shorthand to set a feature to 1
        'ss01', 
        'ss02',
      },
    },
    -- Fallback fonts 
    {
      name = 'Monaspace Argon Var',
      -- Variable font axes
      variations = {
        {
          name = 'slnt',
          value = -11,
        },
      },
    },
    -- Shorthand for no features or variations
    'Symbols Nerd Font',
    'Noto Color Emoji',
  },
  font_size = {
    kind = 'width', -- 'width' | 'height'
    size = 10,
  },
  -- Multipliers of the base animation speed.
  -- To disable animations, set these to large values like 1000.
  cursor_speed = 2,
  scroll_speed = 2,
  -- Increase or decrease the distance from the baseline for underlines.
  underline_offset = 1,
  -- For transparent window effects, use this to set the default background color. 
  -- This is because most colorschemes in transparent mode unset the background,
  -- which normally defaults to the terminal background, but we don't have that here. 
  -- You must also pass --transparent as a command-line argument to see the effect.
  -- Channel values are in the range 0-255. 
  bg_override = {
    r = 48,
    g = 52,
    b = 70,
    a = 128,
  },
})

-- Alternatively, the guifont option is supported:
vim.opt.guifont = 'Cascadia Code PL:w10, Symbols Nerd Font, Noto Color Emoji'

-- There are also freestanding functions to set these options as desired. 
-- Below is a keymap for increasing and decreasing the font size:

-- Increase font size
vim.keymap.set('n', '<c-+>', function()
  neophyte.set_font_width(neophyte.get_font_width() + 1)
end)

-- Decrease font size
vim.keymap.set('n', '<c-->', function()
  neophyte.set_font_width(neophyte.get_font_width() - 1)
end)

-- Neophyte can also record frames to a PNG sequence.
-- You can convert to a video with ffmpeg:
--
-- ffmpeg -framerate 60 -pattern_type glob -i '/my/frames/location/*.png' 
-- -pix_fmt yuv422p -c:v libx264 -vf 
-- "colorspace=all=bt709:iprimaries=bt709:itrc=srgb:ispace=bt709:range=tv:irange=pc"  
-- -color_range 1 -colorspace 1 -color_primaries 1 -crf 23 -y /my/output/video.mp4

-- Start rendering
neophyte.start_render('/directory/to/output/frames/')
-- Stop rendering
neophyte.end_render()
```

### Noice

I recommend using Neophyte with [Noice](https://github.com/folke/noice.nvim) for
best results, unless you choose to disable cursor animations. This is because
Noice externalizes several UI features such that Neovim cedes responsibility
for rendering them, namely the cmdline and messages. Without this, the cursor
tends to jump around in a way that is jarring when combined with animations.
Eventually we may support externalizing these UI elements without a plugin
(this is already partially implemented), but in the meantime, Noice is the
best option. If you want to try Noice without opting in to popup notifications
or the popup cmdline, you can try these settings:

```lua
-- lazy.nvim
{
  'folke/noice.nvim',
  event = 'VeryLazy',
  opts = {
    presets = {
      bottom_search = true,
      command_palette = true,
      long_message_to_split = true,
    },
    lsp = {
      message = {
        view = 'mini',
      },
    },
    messages = {
      view = 'mini',
      view_search = false,
    },
    cmdline = {
      view = 'cmdline',
    },
  },
  dependencies = {
    'MunifTanjim/nui.nvim',
  },
}
```

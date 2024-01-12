local M = {}

-- Doc comments reference:
-- https://luals.github.io/wiki/annotations/

---@class FontFeatureFull
---@field name string The font feature name, such as 'liga', 'calt', or 'ordn'.
---@field value integer The font feature value, usually 0 to disable a feature or 1 to enable it. The string shorthand of FontFeature is available to set a feature to 1.

---@alias FontFeature string | FontFeatureFull

---@class FontVariation
---@field name string The font variation name, such as 'wght', 'wdgh', or 'slnt'.
---@field value number The font variation value.

---@class FontFull
---@field name string The font name. If you don't need features or variations, consider using the string shorthand of Font.
---@field features? FontFeature[]
---@field variations? FontVariation[]

---@alias Font string | FontFull

---@alias FontSizeKind 'width' | 'height'

---@class FontSize
---@field kind FontSizeKind
---@field size number

---@class Color
---@field r number
---@field g number
---@field b number
---@field a number

---@class Config
---@field fonts? Font[]
---@field font_size? FontSize
---@field underline_offset? number
---@field cursor_speed? number
---@field scroll_speed? number
---@field bg_override? Color

---@alias motion 'still' | 'animating'

---Set Neophyte configuration
---@param config Config
function M.setup(config)
  if not M.is_running() then
    return
  end

  vim.api.nvim_create_autocmd('VimLeavePre', {
    callback = function()
      vim.rpcnotify(1, 'neophyte.leave', {})
    end
  })

  if config.fonts ~= nil then
    M.set_fonts(config.fonts)
  end

  if config.font_size ~= nil then
    local font_size = config.font_size
    assert(font_size)
    local kind = font_size.kind
    assert(kind)
    local size = font_size.size
    assert(size)
    if kind == 'width' then
      M.set_font_width(size)
    elseif kind == 'height' then
      M.set_font_height(size)
    end
  end

  if config.underline_offset ~= nil then
    M.set_underline_offset(config.underline_offset)
  end

  if config.cursor_speed ~= nil then
    M.set_cursor_speed(config.cursor_speed)
  end

  if config.scroll_speed ~= nil then
    M.set_scroll_speed(config.scroll_speed)
  end

  if config.bg_override ~= nil then
    local bg = config.bg_override
    assert(bg)
    M.set_bg_override(bg.r, bg.g, bg.b, bg.a)
  end
end

---Gets whether Neovim is running in Neophyte
function M.is_running()
  local success, result = pcall(function() return vim.rpcrequest(1, 'neophyte.is_running', {}) end)
  -- May not be a bool if handled by another frontend
  return success and result == true
end

---Set the height of the font
---@param height number
function M.set_font_height(height)
  vim.rpcnotify(1, 'neophyte.set_font_height', { height })
end

---Get the width of the font
---@return number
function M.get_font_height()
  return vim.rpcrequest(1, 'neophyte.get_font_height', {})
end

---Set the font width
---@param width number
function M.set_font_width(width)
  vim.rpcnotify(1, 'neophyte.set_font_width', { width })
end

---Get the font width
---@return number
function M.get_font_width()
  return vim.rpcrequest(1, 'neophyte.get_font_width', {})
end

---Set the fonts to use, higher-priority fonts coming first and fallbacks after
---@param fonts Font[]
function M.set_fonts(fonts)
  vim.rpcnotify(1, 'neophyte.set_fonts', fonts)
end

---Set the offset of underlines from the font baseline
---@return number
function M.get_underline_offset()
  return vim.rpcrequest(1, 'neophyte.get_underline_offset', {})
end

---Get the offset of underlines from the font baseline
---@param offset number
function M.set_underline_offset(offset)
  vim.rpcnotify(1, 'neophyte.set_underline_offset', { offset })
end

---Get the names of loaded fonts
---@return string[]
function M.get_fonts()
  return vim.rpcrequest(1, 'neophyte.get_fonts', {})
end

---Set the cursor speed as a multiple of the base speed
---@param speed number
function M.set_cursor_speed(speed)
  vim.rpcnotify(1, 'neophyte.set_cursor_speed', { speed })
end

---Get the cursor speed as a multiple of the base speed
---@return number
function M.get_cursor_speed()
  return vim.rpcrequest(1, 'neophyte.get_cursor_speed', {})
end

---Set the scroll speed as a multiple of the base speed
---@param speed number
function M.set_scroll_speed(speed)
  vim.rpcnotify(1, 'neophyte.set_scroll_speed', { speed })
end

---Get the scroll speed as a multiple of the base speed
---@return number
function M.get_scroll_speed()
  return vim.rpcrequest(1, 'neophyte.get_scroll_speed', {})
end

---Set the size of the render target in pixels
---@param width integer
---@param height integer
function M.set_render_size(width, height)
  vim.rpcnotify(1, 'neophyte.set_render_size', { width, height })
end

---Undoes the effect of set_render_size such that Neophyte sets the render target size based on the window size.
function M.unset_render_size()
  vim.rpcnotify(1, 'neophyte.unset_render_size', {})
end

---Gets the current size of the render target
---@return { width: integer, height: integer }
function M.get_render_size()
  return vim.rpcrequest(1, 'neophyte.get_render_size', {})
end

---Output rendered frames to the given directory as PNGs. Frames are named with the number of microseconds since the render was started.
---@param directory string
function M.start_render(directory)
  vim.rpcnotify(1, 'neophyte.start_render', { directory })
end

---Stops rendering the directory set by start_render.
function M.end_render()
  vim.rpcnotify(1, 'neophyte.end_render', {})
end

---@param r number The red channel in 0-255
---@param g number The green channel in 0-255
---@param b number The blue channel in 0-255
---@param a number The alpha channel in 0-255
function M.set_bg_override(r, g, b, a)
  vim.rpcnotify(1, 'neophyte.set_bg_override', { r, g, b, a })
end

return M

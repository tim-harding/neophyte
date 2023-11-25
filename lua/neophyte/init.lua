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

---@alias FontSizeKind "width" | "height"

---@class FontSize
---@field kind FontSizeKind
---@field size number

---@class Config
---@field fonts? Font[]
---@field font_size? FontSize
---@field underline_offset? number
---@field cursor_speed? number
---@field scroll_speed? number

---@param config Config
function M.setup(config)
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
end

---@param height number
function M.set_font_height(height)
  vim.rpcnotify(1, "neophyte.set_font_height", { height })
end

---@return number
function M.get_font_height()
  return vim.rpcrequest(1, "neophyte.get_font_height", {})
end

---@param width number
function M.set_font_width(width)
  vim.rpcnotify(1, "neophyte.set_font_width", { width })
end

---@return number
function M.get_font_width()
  return vim.rpcrequest(1, "neophyte.get_font_width", {})
end

---@param fonts Font[]
function M.set_fonts(fonts)
  vim.rpcnotify(1, "neophyte.set_fonts", fonts)
end

---@return number
function M.get_underline_offset()
  return vim.rpcrequest(1, "neophyte.get_underline_offset", {})
end

---@param offset number
function M.set_underline_offset(offset)
  vim.rpcnotify(1, "neophyte.set_underline_offset", { offset })
end

---@return string[]
function M.get_fonts()
  return vim.rpcrequest(1, "neophyte.get_fonts", {})
end

---@param speed number
function M.set_cursor_speed(speed)
  vim.rpcnotify(1, "neophyte.set_cursor_speed", { speed })
end

---@return number
function M.get_cursor_speed()
  return vim.rpcrequest(1, "neophyte.get_cursor_speed", {})
end

---@param speed number
function M.set_scroll_speed(speed)
  vim.rpcnotify(1, "neophyte.set_scroll_speed", { speed })
end

---@return number
function M.get_scroll_speed()
  return vim.rpcrequest(1, "neophyte.get_scroll_speed", {})
end

---@param width integer
---@param height integer
function M.set_render_size(width, height)
  vim.rpcnotify(1, "neophyte.set_render_size", { width, height })
end

function M.unset_render_size()
  vim.rpcnotify(1, "neophyte.unset_render_size", {})
end

---@return integer
function M.get_render_size()
  return vim.rpcrequest(1, "neophyte.get_render_size", {})
end

---@param directory string
function M.start_render(directory)
  vim.rpcnotify(1, "neophyte.start_render", { directory })
end

function M.end_render()
  vim.rpcnotify(1, "neophyte.end_render", {})
end

return M

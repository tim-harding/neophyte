local M = {}

-- Doc comments reference:
-- https://github.com/LuaLS/lua-language-server/wiki/Annotations

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

---@param fonts string[] | { name: string, features?: string[] | { name: string, value: number }[], variations?: string[] | { name: string, value: number }[] }[]
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

return M

local M = {}

-- Doc comments reference:
-- https://github.com/LuaLS/lua-language-server/wiki/Annotations

---@param height number
function M.set_font_height(height)
  local integral, _ = math.modf(height)
  vim.rpcnotify(1, "neophyte.set_font_height", { integral })
end

---@return number
function M.get_font_height()
  return vim.rpcrequest(1, "neophyte.get_font_height", {})
end

---@param width number
function M.set_font_width(width)
  local integral, _ = math.modf(width)
  vim.rpcnotify(1, "neophyte.set_font_width", { integral })
end

---@return number
function M.get_font_width()
  return vim.rpcrequest(1, "neophyte.get_font_width", {})
end

-- TODO
---@param fonts string[]
function M.set_fonts(fonts)
  vim.rpcnotify(1, "neophyte.set_fonts", fonts)
end

---@return string[]
function M.get_fonts()
  return vim.rpcrequest(1, "neophyte.get_fonts", {})
end

-- TODO
---@param speed number
function M.set_cursor_speed(speed)
  vim.rpcnotify(1, "neophyte.set_cursor_speed", speed)
end

---@return number
function M.get_cursor_speed()
  return vim.rpcrequest(1, "neophyte.get_cursor_speed", {})
end

-- TODO
---@param speed number
function M.set_scroll_speed(speed)
  vim.rpcnotify(1, "neophyte.set_scroll_speed", speed)
end

---@return number
function M.get_scroll_speed()
  return vim.rpcrequest(1, "neophyte.get_scroll_speed", {})
end

return M

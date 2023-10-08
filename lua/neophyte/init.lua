local M = {}

---@param height number
function M.set_font_height(height)
  local integral, _ = math.modf(height)
  vim.rpcnotify(1, "neophyte.set_font_height", { integral })
end

---@param width number
function M.set_font_width(width)
  local integral, _ = math.modf(width)
  vim.rpcnotify(1, "neophyte.set_font_width", { integral })
end

return M

local M = {}

---@param height number
function M.set_font_height(height)
  vim.rpcnotify(1, "neophyte.set_font_height", { height })
end

---@param width number
function M.set_font_width(width)
  vim.rpcnotify(1, "neophyte.set_font_width", { width })
end

return M

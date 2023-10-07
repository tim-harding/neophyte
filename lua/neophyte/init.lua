local M = {}

---@param height number
function M.set_font_height(height)
  print('set_font_height')
  vim.rpcnotify(1, "neophyte.set_font_height", { height })
end

return M

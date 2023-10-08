local M = {}

---@param height number
function M.set_font_height(height)
  vim.rpcnotify(1, "neophyte.set_font_height", { height, height })
end

return M

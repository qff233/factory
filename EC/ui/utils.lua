local component = require("component")
local gpu = component.gpu

local utils = {}

---@param x number
---@param y number
---@param width number
---@param height number
function utils.draw_border(x, y, width, height)
    gpu.set(x, y, "┌" .. string.rep("─", width - 2) .. "┐")
    gpu.set(x, y + height - 1, "└" .. string.rep("─", width - 2) .. "┘")
    for i = 1, height - 2 do
        gpu.set(x, y + i, "│")
        gpu.set(x + width - 1, y + i, "│")
    end
end

return utils

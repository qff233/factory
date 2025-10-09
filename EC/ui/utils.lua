local component = require("component.init")
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

function utils.utf8len(input)
    if not input then
        return 0
    end
    local len = string.len(input)
    local left = len
    local cnt = 0
    local arr = {0, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc}
    while left ~= 0 do
        local tmp = string.byte(input, -left)
        local i = #arr
        while arr[i] do
            if tmp >= arr[i] then
                left = left - i
                break
            end
            i = i - 1
        end
        cnt = cnt + 1
    end
    return cnt
end

return utils

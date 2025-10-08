---@class GPU
local gpu = {}

function gpu.getResolution()
    return 80, 50
end

function gpu.getBackground()
    return 0x000000
end

function gpu.getForeground()
    return 0xFFFFFF
end

---@param color number
function gpu.setBackground(color)
    -- print("gpu.setBackground", color)
end

---@param color number
function gpu.setForeground(color)
    -- print("gpu.setForeground", color)
end

---@param x number
---@param y number
---@param width number
---@param height number
---@param char string
function gpu.fill(x, y, width, height, char)
    -- print("gpu.fill", x, y, width, height, char)
end

---@param x number
---@param y number
---@param value number
---@param vertical? boolean
function gpu.set(x, y, value, vertical)
    -- print("gpu.set", x, y, value)
end

return gpu

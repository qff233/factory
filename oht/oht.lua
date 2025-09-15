local M = {}

local interface = require("interface")
local sides = require("sides")

---@class OHT
---@field robot Robot
---@field pos Pos
OHT = {}
OHT.__index = OHT

function OHT.new()
    local self = setmetatable({}, OHT)
    self.robot = interface.get_robot()
    self.pos = { x = 0, y = 0, z = 0 }
    return self
end

---comment
---@param pos Pos
function OHT:move(pos)
    local dx = pos.x - self.pos.x
    local dy = pos.y - self.pos.y
    local dz = pos.z - self.pos.z

    while true do
        local max = 0.0
        if math.abs(dx) > max then
            max = math.abs(dx)
        end
        if math.abs(dy) > max then
            max = math.abs(dy)
        end
        if math.abs(dz) > max then
            max = math.abs(dz)
        end

        if max < 0.5 then
            break
        end

        ---@type Sides
        local side
        if max == math.abs(dx) then
            if dx > 0 then
                side = sides.posx
                dx = dx - 1
            else
                side = sides.negx
                dx = dx + 1
            end
        elseif max == math.abs(dy) then
            if dy > 0 then
                side = sides.posy
                dy = dy - 1
            else
                side = sides.negy
                dy = dy + 1
            end
        elseif max == math.abs(dz) then
            if dz > 0 then
                side = sides.posz
                dz = dz - 1
            else
                side = sides.negz
                dz = dz + 1
            end
        end
        self.robot:move(side)
    end

    self.pos = pos
end

---todo drop fill

function M.new()
    local self = OHT.new()
    return self
end

return M

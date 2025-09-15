local M = {}

---@class Robot
Robot = {}
Robot.__index = Robot

function M.get_robot()
    local self = setmetatable({}, Robot)
    return self
end

---comment
---@param side Sides
local function sides_to_string(side)
    local side_string_map = { "negy", "posy", "negz", "posz", "negx", "posx" }
    return side_string_map[side + 1]
end

---comment
---@param side Sides
---@return boolean
local function move(side)
    print("move " .. sides_to_string(side))
    return true
end

---comment
---@param side Sides
function Robot:move(side)
    while true do
        if move(side) then
            self:set_light_color(0x00FF00)
            break
        end

        self:set_light_color(0xFF0000)
        os.sleep(1)
    end
end

---comment
---@param color number
function Robot:set_light_color(color)
    print("color is " .. color)
end

---comment
---@param slot number
---@param side Sides
---@param count number | nil
function Robot:drop(slot, side, count)
    print("select slot is " .. slot)
    print("side is " .. side)
    print("count is " .. count)
end

---comment
---@param slot number
---@param side Sides
---@param count number | nil
function Robot:fill(slot, side, count)
    print("select slot is " .. slot)
    print("side is " .. side)
    print("count is " .. count)
end

function os.sleep(secounds)
    print("sleep " .. secounds .. "secounds")
end

return M

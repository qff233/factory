local utils = require("ui.utils")
local component = require("component")
local gpu = component.gpu
local Widget = require("ui.widget")

---@class ProgressBar: Widget
---@field value fun(value: number):number
---@field min_value number
---@field max_value number
---@field bar_color number
---@field background_color number
---@field border_color number
local ProgressBar = {}
ProgressBar.__index = setmetatable(ProgressBar, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param value number
---@field bar_color number?
---@field background_color number?
---@field border_color number?
---@return ProgressBar
function ProgressBar.new(x, y, width, height, value, bar_color, background_color, border_color)
    ---@type ProgressBar
    local self = setmetatable(Widget.new(x, y, width, height), ProgressBar)
    self.value = WATCHABLE(value or 0.0).set(function()
        self:set_dirty()
    end)
    self.bar_color = bar_color or 0x00FF01
    self.background_color = background_color or 0x333333
    self.border_color = border_color or 0x666666
    return self
end

function ProgressBar:on_draw()
    local value = self.value()
    local x, y = self:get_absolute_xy()
    local bar_width = math.floor((self.width - 2) * value)

    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height)

    gpu.setForeground(self.border_color)

    utils.draw_border(x, y, self.width, self.height)

    if bar_width > 0 then
        gpu.setBackground(self.bar_color)
        for cy = y + 1, y + self.height - 2 do
            gpu.set(x + 1, cy, string.rep(" ", bar_width))
        end
    end
end

return ProgressBar

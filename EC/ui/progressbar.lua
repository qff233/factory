local utils = require("ui.utils")
local component = require("component")
local gpu = component.gpu
local Widget = require("ui.widget")

---@class ProgressBar: Widget
---@field value fun(value: number):number
---@field text fun(value: string):string
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
function ProgressBar.new(x, y, width, height, value, text, bar_color, background_color, border_color)
    ---@type ProgressBar
    local self = setmetatable(Widget.new(x, y, width, height), ProgressBar)
    self.value = WATCHABLE(value or 0.0).set(function()
        self:set_dirty()
    end)
    self.text = WATCHABLE(text or 0.0).set(function()
        self:set_dirty()
    end)
    self.bar_color = bar_color or 0x00FF01
    self.background_color = background_color or 0x000000
    self.border_color = border_color or 0x666666
    return self
end

function ProgressBar:on_draw()
    local value = self.value()
    local x, y = self:get_absolute_xy()
    local text = self.text()
    local bar_width = math.floor((self.width - 2) * value)
    local text_width = utils.utf8len(text)
    local start_text_idx = math.floor((self.width - text_width) / 2) + x

    print(start_text_idx, text_width)

    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height, " ")

    gpu.setForeground(self.border_color)
    utils.draw_border(x, y, self.width, self.height)

    if bar_width > 0 then
        gpu.setBackground(self.bar_color)
        for cy = y + 1, y + self.height - 2 do
            for cx = x + 1, x + self.width - 2 do
                if cx > x + bar_width then
                    gpu.setBackground(self.background_color)
                end
                if cx >= start_text_idx and cx < start_text_idx + text_width then
                    local text_idx = cx - start_text_idx + 1
                    gpu.set(cx, cy, string.sub(text, text_idx, text_idx))
                elseif cx <= x + bar_width then
                    gpu.set(cx, cy, " ")
                end
            end
        end
    end
end

return ProgressBar

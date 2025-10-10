local component = require("component")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class Panel: Widget
---@field title string
---@field border_color number
---@field background_color number
local Panel = {}
Panel.__index = setmetatable(Panel, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param title string?
---@param border_color number?
---@param background_color number?
function Panel.new(x, y, width, height, title, border_color, background_color)
    ---@type Panel
    local self = setmetatable(Widget.new(x, y, width, height), Panel)
    self.title = title
    self.border_color = border_color or 0xFFFFFF
    self.background_color = background_color or 0x000000
    return self
end

function Panel:on_draw()
    local x, y = self:get_absolute_xy()

    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height, " ")

    gpu.setForeground(self.border_color)
    utils.draw_border(x, y, self.width, self.height)

    if utils.utf8len(self.title) > 0 then
        local title_x = x + math.floor((self.width - utils.utf8len(self.title)) / 2)
        gpu.set(title_x, y, " " .. self.title .. " ")
    end
end

return Panel

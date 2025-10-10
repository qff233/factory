local component = require("component")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class Button: Widget
---@field text fun(text: string)
---@field text_color fun(text_color: number)
---@field current_background_color number
---@field background_color number
---@field pressed_color number
---@field is_clicked boolean
---@field on_clicked fun()
local Button = {}
Button.__index = setmetatable(Button, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param on_clicked fun()
---@param text string?
---@param text_color number?
---@param background_color number?
---@field pressed_color number?
function Button.new(x, y, width, height, on_clicked, text, text_color, background_color, pressed_color)
    ---@type Button
    local self = setmetatable(Widget.new(x, y, width, height), Button)
    self.text = WATCHABLE(text or "").set(function()
        self:set_dirty()
    end)
    self.text_color = WATCHABLE(text_color or 0xFFFFFF).set(function()
        self:set_dirty()
    end)
    self.current_background_color = background_color or 0x333333
    self.background_color = background_color or 0x333333
    self.pressed_color = pressed_color or 0x666666
    self.on_clicked = on_clicked
    self.is_clicked = false
    return self
end

function Button:on_draw()
    local text = self.text()
    local text_color = self.text_color()

    local x, y = self:get_absolute_xy()
    gpu.setBackground(self.current_background_color)
    gpu.fill(x, y, self.width, self.height, " ")

    -- 渲染字体
    gpu.setForeground(text_color)
    utils.draw_border(x, y, self.width, self.height)
    local x = x + math.floor((self.width - utils.utf8len(text)) / 2)
    local y = y + math.floor((self.height - 1) / 2)
    x = math.max(x, math.min(x, x + self.width - 1))

    if utils.utf8len(text) > self.width then
        text = string.sub(text, 1, self.width)
    end
    gpu.set(x, y, text)
end

function Button:parse_event(event)
    if event[1] == "touch" then
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x, event_y) then
            self:set_dirty()
            self.current_background_color = self.pressed_color
            self.on_clicked()
            self.is_clicked = true
            return true
        end
    elseif event[1] == "drop" and self.is_clicked then
        self.is_clicked = false
        self:set_dirty()
        self.current_background_color = self.background_color
        return true
    end
    return false
end

return Button

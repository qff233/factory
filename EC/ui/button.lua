local gpu = require("component.gpu")
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class Button: Widget
---@field text fun(text: string)
---@field text_color fun(text_color: number)
---@field current_background_color number
---@field background_color number
---@field pressed_color number
---@field on_clicked fun()
local Button = {}
Button.__index = setmetatable(Button, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param text string
---@param text_color number
---@param background_color number
---@field pressed_color number
---@param on_clicked fun()
function Button.new(x, y, width, height, on_clicked, text, text_color, background_color, pressed_color)
    ---@type Button
    local self = setmetatable(Widget.new(x, y, width, height), Button)
    self.text = WATCHABLE(text or "").set(function()
        self.dirty = true
    end)
    self.text_color = WATCHABLE(text_color or 0xFFFFFF).set(function()
        self.dirty = true
    end)
    self.current_background_color = background_color or 0x333333
    self.background_color = background_color or 0x333333
    self.pressed_color = pressed_color or 0x222222
    self.on_clicked = on_clicked
    return self
end

function Button:on_draw()
    local text = self.text()
    local text_color = self.text_color()

    gpu.setBackground(self.current_background_color)
    gpu.fill(self.x, self.y, self.width, self.height, " ")

    gpu.setForeground(text_color)
    local x = self.x + math.floor((self.width - #text) / 2)
    local y = self.y + math.floor((self.height - 1) / 2)
    x = math.max(x, math.min(x, x + self.width - 1))

    if #text > self.width then
        text = string.sub(text, 1, self.width)
    end
    utils.draw_border(self.x, self.y, self.width, self.height)
    gpu.set(x, y, text)
end

function Button:parse_event(event)
    if event[1] == "touch" then
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x + 1, event_y + 1) then
            self.dirty = true
            self.current_background_color = self.pressed_color
            self.on_clicked()
            return true
        end
    elseif event[1] == "drop" then
        local event_x, event_y = event[3], event[4]
        self.dirty = true
        self.current_background_color = self.background_color
    end
    return false
end

return Button

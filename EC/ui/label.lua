local component = require("component.init")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class Label: Widget
---@field text fun(text: string)
---@field text_color fun(text_color: number)
---@field background_color fun(background_color: number)
local Label = {}
Label.__index = setmetatable(Label, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param text string?
---@param text_color number?
---@param background_color number?
function Label.new(x, y, width, height, text, text_color, background_color)
    ---@type Label
    local self = setmetatable(Widget.new(x, y, width, height), Label)
    self.text = WATCHABLE(text or "").set(function()
        self.dirty = true
    end)
    self.text_color = WATCHABLE(text_color or 0xFFFFFF).set(function()
        self.dirty = true
    end)
    self.background_color = WATCHABLE(background_color).set(function()
        self.dirty = true
    end)
    return self
end

function Label:on_draw()
    local text = self.text()
    local text_color = self.text_color()
    local background_color = self.background_color()

    if not text or utils.utf8len(text) == 0 then
        return
    end

    local x, y = self:get_absolute_xy()

    if background_color then
        gpu.setBackground(background_color)
        gpu.fill(x, y, self.width, self.height, " ")
    end

    gpu.setForeground(text_color)
    local x = x + math.floor((self.width - utils.utf8len(text)) / 2)
    local y = y + math.floor((self.height - 1) / 2)
    x = math.max(x, math.min(x, x + self.width - 1))

    if utils.utf8len(text) > self.width then
        text = string.sub(text, 1, self.width)
    end

    gpu.set(x, y, text)
end

return Label

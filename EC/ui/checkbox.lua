local component = require("component")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class CheckBox : Widget
---@field text string
---@field text_color number
---@field background_color number
---@field unchecked_color number
---@field checked_color number
---@field is_checked boolean
---@field on_change fun(value: boolean)
local CheckBox = {}
CheckBox.__index = setmetatable(CheckBox, Widget)

---@param x number
---@param y number
---@param on_change fun(value: boolean)
---@param is_checked boolean?
---@param text string?
---@param text_color number?
---@param backgroud_color number?
---@param unchecked_color number?
---@param checked_color number?
function CheckBox.new(x, y, on_change, is_checked, text, text_color, backgroud_color, unchecked_color, checked_color)
    ---@type CheckBox
    local self = setmetatable(Widget.new(x, y, 3 + utils.utf8len(text)), CheckBox)
    self.on_change = on_change
    self.text = text or ""
    self.text_color = text_color or 0xFFFFFF
    self.background_color = backgroud_color or 0x000000
    self.unchecked_color = unchecked_color or 0x666666
    self.checked_color = checked_color or 0x00FF00
    self.is_checked = is_checked or false
    self.on_change = on_change
    return self
end

function CheckBox:on_draw()
    local x, y = self:get_absolute_xy()

    gpu.setBackground(self.background_color)
    gpu.set(x, y, string.rep(" ", self.width))

    gpu.setForeground(self.is_checked and self.checked_color or self.unchecked_color)
    gpu.set(x, y, self.is_checked and "[X]" or "[ ]")

    gpu.setForeground(self.text_color)
    gpu.set(x + 4, y, self.text)
end

function CheckBox:parse_event(event)
    if event[1] == "touch" then
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x, event_y) then
            self:set_dirty()
            self.is_checked = not self.is_checked
            self.on_change(self.is_checked)
            return true
        end
    end
    return false
end

return CheckBox

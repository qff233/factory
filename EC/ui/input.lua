local utils = require("ui.utils")
local gpu = require("component.gpu")
local Widget = require("ui.widget")

---@class Input: Widget
---@field text string
---@field text_color number
---@field background_color number
---@field is_focused boolean
---@field cursor_pos number
---@field on_submit fun(text: string)
local Input = {}
Input.__index = setmetatable(Input, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param text string
---@param text_color number
---@param background_color number
function Input.new(x, y, width, height, on_submit, text, text_color, background_color)
    ---@type Input
    local self = setmetatable(Widget.new(x, y, width, height), Input)
    self.on_submit = on_submit
    self.text = text or ""
    self.text_color = text_color or 0xFFFFFF
    self.background_color = background_color or 0x000000
    self.is_focused = false
    self.cursor_pos = 1
    return self
end

function Input:on_draw()
    local max_length = self.width - 2

    gpu.setBackground(self.background_color)
    gpu.fill(self.x, self.y, self.width, self.height, " ")

    if self.is_focused then
        gpu.setForeground(self.text_color)
    else
        gpu.setForeground(0x666666)
    end
    utils.draw_border(self.x, self.y, self.width, self.height)

    -- 绘制的文本
    local text_start = 1
    local draw_text = self.text
    if #draw_text > max_length then
        if self.cursor_pos > max_length then
            text_start = self.cursor_pos - max_length + 1
        end
        draw_text = string.sub(draw_text, text_start, text_start + max_length - 1)
    end

    local x = self.x + 1
    local y = self.y + math.floor((self.height - 1) / 2)
    gpu.set(x, y, draw_text)

    -- 绘制光标
    if self.is_focused then
        local cursor_x = x + (self.cursor_pos - text_start)
        if cursor_x <= x + self.cursor_pos - text_start then
            gpu.setBackground(self.text_color)
            local cursor_char = " "
            if cursor_x - x <= #self.text then
                cursor_char = string.sub(self.text, cursor_x - x, cursor_x - x)
            end
            gpu.set(cursor_x, y, cursor_char)
        end
    end
end

function Input:parse_event(event)
    if event[1] == "touch" then
        self.dirty = true
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x + 1, event_y + 1) then
            self.is_focused = true
            local relative_x = event_x - self.x
            self.cursor_pos = math.min(relative_x, #self.text + 1)
            return true
        else
            self.is_focused = false
        end
    elseif self.is_focused and event[1] == "key_down" then
        self.dirty = true
        local char, code = event[3], event[4]
        local max_length = self.width - 2
        if code == 0x0E then -- 退格键
            if self.cursor_pos > 1 then
                self.text = string.sub(self.text, 1, self.cursor_pos - 2) .. string.sub(self.text, self.cursor_pos)
            end
        elseif code == 0xCB then -- 左键头
            self.cursor_pos = math.max(1, self.cursor_pos - 1)
        elseif code == 0xCD then -- 右键头
            self.cursor_pos = math.min(#self.text + 1, self.cursor_pos + 1)
        elseif code == 0x1C then -- 回车键
            self.on_submit(self.text)
        else -- 字符
            if #self.text < max_length then
                self.text = string.sub(self.text, 1, self.cursor_pos - 1) .. char ..
                                string.sub(self.text, self.cursor_pos)
                self.cursor_pos = self.cursor_pos + 1
            end
        end
        return true
    end
    return false
end

return Input

local utils = require("ui.utils")
local component = require("component")
local gpu = component.gpu
local Widget = require("ui.widget")
local unicode = require("unicode")

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
---@param on_submit fun(self: Input)
---@param text string?
---@param text_color number?
---@param background_color number?
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

local function get_display_width(text)
    local width = 0
    for i = 1, unicode.len(text) do
        local char = unicode.sub(text, i, i)
        width = width + unicode.charWidth(char)
    end
    return width
end

---@param text string
---@param cursor_char_pos number 光标在文本中的位置（按字符数，从1开始）
---@param max_display_width number 输入框可用的最大显示宽度（即 self.width - 2）
---@return number display_x 光标的相对显示X坐标（从文本框内容区左边缘开始，按字符单元格计算）
---@return number start_char_index 绘制的文本子串的起始字符索引（用于水平滚动）
local function get_cursor_display_info(text, cursor_char_pos, max_display_width)
    local total_display_width = get_display_width(text)
    local prefix_text = unicode.sub(text, 1, cursor_char_pos - 1)
    local cursor_display_x = get_display_width(prefix_text) + 1

    local start_char_index = 1

    if cursor_display_x > max_display_width then
        local current_width = 0
        start_char_index = cursor_char_pos

        while start_char_index > 1 and current_width < max_display_width do
            start_char_index = start_char_index - 1
            local char = unicode.sub(text, start_char_index, start_char_index)
            local char_width = unicode.charWidth(char)

            if current_width + char_width > max_display_width then
                start_char_index = start_char_index + 1
                break
            end

            current_width = current_width + char_width
        end
        prefix_text = unicode.sub(text, start_char_index, cursor_char_pos - 1)
        cursor_display_x = get_display_width(prefix_text) + 1
    end

    if cursor_display_x > max_display_width then
        cursor_display_x = max_display_width
    end

    return cursor_display_x, start_char_index
end

function Input:on_draw()
    local max_display_width = self.width - 2

    local x, y = self:get_absolute_xy()
    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height, " ")

    if self.is_focused then
        gpu.setForeground(self.text_color)
    else
        gpu.setForeground(0x666666)
    end
    utils.draw_border(x, y, self.width, self.height)

    local cursor_display_x, start_char_index = get_cursor_display_info(self.text, self.cursor_pos, max_display_width)
    local draw_text = ""
    local current_width = 0
    local end_char_index = start_char_index

    while end_char_index <= unicode.len(self.text) and current_width < max_display_width do
        local char = unicode.sub(self.text, end_char_index, end_char_index)
        local char_width = unicode.charWidth(char)

        if current_width + char_width <= max_display_width then
            draw_text = draw_text .. char
            current_width = current_width + char_width
            end_char_index = end_char_index + 1
        else
            break
        end
    end

    -- 绘制文本
    local text_x = x + 1
    local text_y = y + math.floor((self.height - 1) / 2)
    gpu.set(text_x, text_y, draw_text)

    -- 绘制光标
    if self.is_focused then
        local cursor_absolute_x = text_x + cursor_display_x - 1
        if cursor_absolute_x >= text_x and cursor_absolute_x <= text_x + max_display_width then
            gpu.setBackground(self.text_color)
            gpu.set(cursor_absolute_x, text_y, " ")
        end
    end
end

function Input:parse_event(event)
    if event[1] == "touch" then
        self:set_dirty()
        local event_x, event_y = event[3], event[4]
        local x, y = self:get_absolute_xy()
        if self:contains(event_x, event_y) then
            self.is_focused = true
            local relative_x = event_x - x - 1
            if relative_x < 0 then
                relative_x = 0
            end

            local current_width = 0
            local target_pos = 1
            for i = 1, unicode.len(self.text) do
                local char = unicode.sub(self.text, i, i)
                local char_width = unicode.charWidth(char)

                if relative_x >= current_width and relative_x < current_width + char_width then
                    if char_width == 2 and relative_x - current_width < 1 then
                        target_pos = i
                    else
                        target_pos = i + 1
                    end
                    break
                end

                current_width = current_width + char_width
                if i == unicode.len(self.text) then
                    target_pos = unicode.len(self.text) + 1
                end
            end
            self.cursor_pos = target_pos
            return true
        else
            self.is_focused = false
        end
    elseif self.is_focused and event[1] == "key_down" then
        self:set_dirty()
        local char, code = event[3], event[4]
        local max_display_width = self.width - 2
        local current_text_display_width = get_display_width(self.text)

        if code == 0x0E then -- 退格键
            if self.cursor_pos > 1 then
                self.text = unicode.sub(self.text, 1, self.cursor_pos - 2) .. unicode.sub(self.text, self.cursor_pos)
                self.cursor_pos = self.cursor_pos - 1
            end
        elseif code == 0xCB then -- 左键头
            self.cursor_pos = math.max(1, self.cursor_pos - 1)
        elseif code == 0xCD then -- 右键头
            self.cursor_pos = math.min(unicode.len(self.text) + 1, self.cursor_pos + 1)
        elseif code == 0x1C then -- 回车键
            self.on_submit(self)
        else -- 字符
            local input_char = unicode.char(char)
            local input_char_width = unicode.charWidth(char)
            if current_text_display_width + input_char_width <= max_display_width then
                self.text = unicode.sub(self.text, 1, self.cursor_pos - 1) .. input_char ..
                                unicode.sub(self.text, self.cursor_pos)
                self.cursor_pos = self.cursor_pos + 1
            end
        end
        return true
    end
    return false
end

return Input

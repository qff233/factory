local component = require("component")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")
local Unicode = require("unicode")

---@class List: Widget
---@field items fun(items: string[]):string[]
---@field on_select fun(selected_item: string)
---@field text_color number
---@field background_color number
---@field scroll_offset number
---@field selected_index number
local List = {}
List.__index = setmetatable(List, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param items table
---@param on_select fun(selected_item: string)
---@param test_color number
---@param background_color number
---@return List
function List.new(x, y, width, height, items, on_select, text_color, background_color)
    local self = setmetatable(Widget.new(x, y, width, height), List)
    self.items = WATCHABLE(items).set(function()
        self:set_dirty()
    end)
    self.on_select = on_select
    self.text_color = text_color or 0xFFFFFF
    self.background_color = background_color or 0x000000
    self.scroll_offset = 0
    return self
end

---@return string
function List:get_item()
    local items = self.items()
    if items and self.selected_index and items[self.selected_index] then
        return items[self.selected_index]
    else
        return nil
    end
end

function List:on_draw()
    local x, y = self:get_absolute_xy()
    local items = self.items()

    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height, " ")

    gpu.setForeground(self.text_color)
    utils.draw_border(x, y, self.width, self.height)

    local visible_items = math.min(#items - self.scroll_offset, self.height - 2)

    if self.selected_index then
        gpu.setBackground(0x333333)
        local y_pos = y + self.selected_index - self.scroll_offset
        if y_pos > y and y_pos < y + self.height then
            gpu.fill(x + 1, y_pos, self.width - 2, 1, " ")
        end
        gpu.setBackground(self.background_color)
    end

    for i = 1, visible_items do
        local item_index = i + self.scroll_offset
        local item = items[item_index]
        local y_pos = y + i

        gpu.setForeground(self.text_color)
        if Unicode.len(item) > self.width - 3 then
            item = Unicode.sub(item, 1, self.width - 3) .. "…"
        end
        gpu.set(x + 2, y_pos, item)
    end
end

function List:parse_event(event)
    local x, y = self:get_absolute_xy()
    local items = self.items()

    if event[1] == "touch" then
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x, event_y) then
            local relative_y = event_y - y
            if relative_y >= 1 and relative_y <= #items - self.scroll_offset then
                self.selected_index = relative_y + self.scroll_offset
                self.on_select(items[self.selected_index])
                self:set_dirty()
                return true
            end
        end
    elseif event[1] == "scroll" then
        self:set_dirty()
        local direction = event[5]
        if direction > 0 then
            -- 向上滚动
            self.scroll_offset = math.max(0, self.scroll_offset - 1)
        else
            -- 向下滚动
            self.scroll_offset = math.min(math.max(#items - (self.height - 2), 0), self.scroll_offset + 1)
        end
        return true
    end

    return false
end

return List

local component = require("component")
local gpu = component.gpu
local utils = require("ui.utils")
local Widget = require("ui.widget")

---@class List: Widget
---@field items string[]
---@field on_select fun(selected_item: string)
---@field text_color number
---@field background_color number
---@field scroll_offset number
local List = {}
List.__index = setmetatable(List, Widget)

---@param x number
---@param y number
---@param width number
---@param height number
---@param items table
---@param on_select fun(selected_item: any)
---@param test_color number
---@param background_color number
---@return List
function List.new(x, y, width, height, items, on_select, text_color, background_color)
    local self = setmetatable(Widget.new(x, y, width, height), List)
    self.items = items
    self.on_select = on_select
    self.text_color = text_color or 0xFFFFFF
    self.background_color = background_color or 0x000000
    self.scroll_offset = 0
    return self
end

function List:on_draw()
    local x, y = self:get_absolute_xy()

    gpu.setBackground(self.background_color)
    gpu.fill(x, y, self.width, self.height)

    gpu.setForeground(0x666666)
    utils.draw_border(x, y, self.width, self.height)

    local visible_items = math.min(#self.items - self.scroll_offset, self.height - 2)

    for i = 1, visible_items do
        local item_index = i + self.scroll_offset
        local item = self.items[item_index]
        local y_pos = y + i

        gpu.setForeground(self.text_color)
        if utils.utf8len(item) > self.width - 3 then
            item = string.sub(item, 1, self.width - 3) .. "…"
        end
        gpu.set(x + 2, y_pos, item)
    end
end

function List:parse_event(event)
    local x, y = self:get_absolute_xy()

    if event[1] == "touch" then
        local event_x, event_y = event[3], event[4]
        if self:contains(event_x, event_y) then
            local relative_y = event_y - y
            if relative_y >= 1 and relative_y <= #self.items - self.scroll_offset then
                local selected_index = relative_y + self.scroll_offset
                self.on_select(self.items[selected_index])
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
            self.scroll_offset = math.min(#self.items - (self.height - 2), self.scroll_offset + 1)
        end
        return true
    end

    return false
end

return List

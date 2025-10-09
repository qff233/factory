---@class Widget
---@field x number
---@field y number
---@field width number
---@field height number
---@field dirty boolean
---@field visible boolean 绘制
---@field enabled boolean  事件接收
---@field parent Widget?
---@field children Widget[]
local Widget = {}
Widget.__index = Widget

---@param x number
---@param y number
---@param width number
---@param height number
function Widget.new(x, y, width, height)
    ---@type Widget
    local self = setmetatable({}, Widget)
    self.x = x or 1
    self.y = y or 1
    self.width = width or 10
    self.height = height or 1
    self.dirty = true
    self.visible = true
    self.enabled = true
    self.children = {}
    return self
end

function Widget:draw()
    if not self.visible then
        return
    end
    if self.dirty then
        self.dirty = false
        self:on_draw()
    end
    for _, child in ipairs(self.children) do
        child:draw()
    end
end

function Widget:handle_evnet(event)
    if not self.visible or not self.enabled or self:parse_event(event) then
        return false
    end

    for i = #self.children, 1, -1 do
        local child = self.children[i]
        if child:handle_evnet(event) then
            return true
        end
    end
end

---@param widget Widget
function Widget:add_child(widget)
    table.insert(self.children, widget)
    widget.parent = self
end

function Widget:contains(x, y)
    local sx, sy = self:get_absolute_xy()
    return x >= sx and x <= sx + self.width - 1 and y >= sy and y <= sy + self.height - 1
end

function Widget:get_absolute_xy()
    if self.parent then
        local x, y = self.parent:get_absolute_xy()
        return self.x + x - 1, self.y + y - 1
    else
        return self.x, self.y
    end
end

function Widget:disable_visible()
    self.visible = false
    self.dirty = true
    if self.parent then
        self.parent.dirty = true
        for _, widget in ipairs(self.parent.children) do
            widget.dirty = true
        end
    end
end

function Widget:enable_visible()
    self.visible = true
    self.dirty = true
    if self.parent then
        self.parent.dirty = true
        for _, widget in ipairs(self.parent.children) do
            widget.dirty = true
        end
    end
end

--- 返回true停止事件的传递
function Widget:parse_event(event)
    return false
end

function Widget:on_draw()
end

function WATCHABLE(value)
    local watcher
    local value = value
    local proxy = {}
    proxy.set = function(callback)
        watcher = callback
        return setmetatable({}, {
            __call = function(_, new_value)
                if new_value ~= nil then
                    local old_value = value
                    if new_value ~= old_value then
                        value = new_value
                        watcher(newValue, oldValue)
                    end
                end
                return value
            end
        })
    end
    return proxy
end

return Widget

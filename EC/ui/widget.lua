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
    return x >= self.x and x <= self.x + self.width - 1 and y >= self.y and y <= self.y + self.height - 1
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

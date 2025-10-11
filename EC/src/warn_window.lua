local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")

local WarnWindow = {}

---@param parent_widget Widget
---@param x number
---@param y number
---@param width number
---@param height number
---@param warn_label string
function WarnWindow.newUI(parent_widget, x, y, width, height, warn_msg)
    local warn_panel = Panel.new(x, y, width, height, "Warn")
    warn_panel:set_id("warn_window")

    local warn_label = Label.new(2, math.floor((height - 3) / 2), width - 2, 1, warn_msg)
    warn_panel:add_child(warn_label)

    local ok_button = Button.new(math.floor((width - 4) / 2) + 1, height - 3, 4, 3, function()
        parent_widget:del_child("warn_window")
        parent_widget:enable_child_event()
    end, "Ok", 0xFFFFFF, 0x00FF00)
    warn_panel:add_child(ok_button)

    parent_widget:add_child(warn_panel)
end

return WarnWindow

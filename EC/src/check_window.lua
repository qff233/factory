local Panel = require("ui.panel")
local Button = require("ui.button")

local CheckWindow = {}

function CheckWindow.newUI(parent_widget, x, y, width, height, check_callback)
    local check_panel = Panel.new(x, y, width, height, "CheckDel")
    check_panel:set_id("check_window")

    local cancel_button = Button.new(3, height - 3, 8, 3, function()
        parent_widget:del_child("check_window")
        parent_widget:enable_child_event()
    end, "Cancel", 0xFFFFFF, 0x00FF00)
    check_panel:add_child(cancel_button)

    local confirm_button = Button.new(width - 9, height - 3, 9, 3, function()
        check_callback()
        parent_widget:del_child("check_window")
        parent_widget:enable_child_event()
    end, "Confirm", 0xFFFFFF, 0xFF0000)
    check_panel:add_child(confirm_button)

    parent_widget:add_child(check_panel)
end

return CheckWindow

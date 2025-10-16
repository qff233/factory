local Widget = require("ui.widget")
local Panel = require("ui.panel")
local Button = require("ui.button")

local ProcessControl = require("src.process_control")

local ConfigState = {}

function ConfigState.newUI(ec_panel)
    local panel = Panel.new(31, 2, 20, 24, "ChangeState")
    panel:set_id("config_state")

    local online_remote_button = Button.new(3, 4, 16, 3, function()
        ProcessControl.set_online_remote()
        ec_panel:del_child("config_state")
        ec_panel:enable_child_event()
    end, "OnlineRemote")
    panel:add_child(online_remote_button)

    local online_local_button = Button.new(3, 8, 16, 3, function()
        ProcessControl.set_online_local()
        ec_panel:del_child("config_state")
        ec_panel:enable_child_event()
    end, "OnlineLocal")
    panel:add_child(online_local_button)

    local offline_button = Button.new(3, 12, 16, 3, function()
        ProcessControl.set_offline()
        ec_panel:del_child("config_state")
        ec_panel:enable_child_event()
    end, "Offline")
    panel:add_child(offline_button)

    local quit_button = Button.new(4, 21, 14, 3, function()
        ec_panel:del_child("config_state")
        ec_panel:enable_child_event()
    end, "Quit")
    panel:add_child(quit_button)

    ec_panel:add_child(panel)
end

return ConfigState

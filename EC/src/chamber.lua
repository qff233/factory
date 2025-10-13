local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local List = require("ui.list")

local component = require("component")
local GT = component.gt_machine
local Config = require("src.config")
local WarnWindow = require("src.warn_window")
local ProgressContrl = require("src.process_control")

local Chamber = {}

local function chamber_config_panel(parent_widget, id)
    local pannel = Panel.new(21, 1, 40, 25, "输入仓 " .. tostring(id) .. " 号配置面板")
    pannel:set_id("panel")

    local config = Config.load()

    local list = List.new(7, 2, 27, 20, config:get_chamber(id), function()
    end)
    pannel:add_child(list)

    local input = Input.new(7, 22, 21, 3, function(self)
        local item_name = self.text
        local result, msg = config:check_in_input_item(item_name)
        if result then
            pannel:disable_child_event()
            WarnWindow.newUI(pannel, -14, 7, 68, 9, msg)
        else
            config:add_chamber_item(id, item_name)
            list.items(config:get_chamber(id))
        end
    end)
    pannel:add_child(input)

    local del_button = Button.new(29, 22, 5, 3, function()
        local item_name = list:get_item()
        if item_name then
            config:del_chamber_item(id, list:get_item())
            list.items(config:get_chamber(id))
        else
            pannel:disable_child_event()
            WarnWindow.newUI(pannel, -14, 7, 68, 9, "必须选择一个流体")
        end
    end, "Del", 0xFFFFFF, 0xFF0000)
    pannel:add_child(del_button)

    local cancel_button = Button.new(35, 18, 4, 3, function()
        parent_widget:del_child("panel")
        parent_widget:enable_child_event()
    end, "NG")
    pannel:add_child(cancel_button)

    local ok_button = Button.new(35, 22, 4, 3, function()
        local result, msg = config:save()
        if result then
            ProgressContrl.realod_config()
            parent_widget:del_child("panel")
            parent_widget:enable_child_event()
        else
            pannel:disable_child_event()
            WarnWindow.newUI(pannel, -14, 7, 68, 9, msg)
        end
    end, "Ok")
    pannel:add_child(ok_button)

    parent_widget:add_child(pannel)
end

local chamber_panel = Panel.new(1, 5, 40, 21, "舱室")

local chamber1_button = Button.new(11, 14, 10, 5, nil, "C1", 0x000000)
local chamber2_button = Button.new(21, 14, 10, 5, nil, "C2", 0x000000)
local chamber3_button = Button.new(11, 9, 10, 5, nil, "C3", 0x000000)
local chamber4_button = Button.new(21, 9, 10, 5, nil, "C4", 0x000000)
local chamber5_button = Button.new(11, 4, 10, 5, nil, "C5", 0x000000)
local chamber6_button = Button.new(21, 4, 10, 5, nil, "C6", 0x000000)

chamber_panel:add_child(chamber1_button)
chamber_panel:add_child(chamber2_button)
chamber_panel:add_child(chamber3_button)
chamber_panel:add_child(chamber4_button)
chamber_panel:add_child(chamber5_button)
chamber_panel:add_child(chamber6_button)

function Chamber.newUI(ec_panel)
    chamber1_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 1)
    end
    chamber2_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 2)
    end
    chamber3_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 3)
    end
    chamber4_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 4)
    end
    chamber5_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 5)
    end
    chamber6_button.on_clicked = function()
        ec_panel:disable_child_event()
        chamber_config_panel(ec_panel, 6)
    end

    ec_panel:add_child(chamber_panel)
end

function Chamber.getChamberButton()
    return {chamber1_button, chamber2_button, chamber3_button, chamber4_button, chamber5_button, chamber6_button}
end

return Chamber

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

local Chamber = {}

---@param config Config
local function chamber_slot_label_input(chamber, parent_widget, id)
    local slot1_label = Label.new(3, 4 + 3 * (id - 1), 2, 1, tostring(id) .. ":")
    parent_widget:add_child(slot1_label)
    local slot1_input = Input.new(6, 3 + 3 * (id - 1), 25, 3, function()
    end, tostring(chamber[id]))
    parent_widget:add_child(slot1_input)
    return slot1_input
end

local function chamber_config_panel(parent_widget, id)
    local pannel = Panel.new(21, 1, 40, 25, "Input " .. tostring(id) .. " Config Panel")
    pannel:set_id("panel")

    local config = Config.load()

    local chamber = config:get_chamber(id)
    local slot1 = chamber_slot_label_input(chamber, pannel, 1)
    local slot2 = chamber_slot_label_input(chamber, pannel, 2)
    local slot3 = chamber_slot_label_input(chamber, pannel, 3)
    local slot4 = chamber_slot_label_input(chamber, pannel, 4)
    local slot5 = chamber_slot_label_input(chamber, pannel, 5)
    local slot6 = chamber_slot_label_input(chamber, pannel, 6)
    local slot7 = chamber_slot_label_input(chamber, pannel, 7)

    local cancel_button = Button.new(33, 18, 6, 3, function()
        parent_widget:del_child("panel")
        parent_widget:enable_child_event()
    end, "NG")
    pannel:add_child(cancel_button)

    local ok_button = Button.new(33, 22, 6, 3, function()
        chamber[1] = slot1.text
        chamber[2] = slot2.text
        chamber[3] = slot3.text
        chamber[4] = slot4.text
        chamber[5] = slot5.text
        chamber[6] = slot6.text
        chamber[7] = slot7.text

        local result, msg = config:set_chamber(id, chamber)
        if result then
            config:save()
            parent_widget:del_child("panel")
            parent_widget:enable_child_event()
        else
            parent_widget:disable_child_event()
            WarnWindow.newUI(parent_widget, 2, 7, 68, 9, msg)
        end
    end, "Ok", 0xFFFFFF, 0xFF0000)
    pannel:add_child(ok_button)

    parent_widget:add_child(pannel)
end

local chamber_panel = Panel.new(1, 5, 40, 21, "chamber")

local chamber1_button = Button.new(11, 14, 10, 5, nil, "C1")
local chamber2_button = Button.new(21, 14, 10, 5, nil, "C2")
local chamber3_button = Button.new(11, 9, 10, 5, nil, "C3")
local chamber4_button = Button.new(21, 9, 10, 5, nil, "C4")
local chamber5_button = Button.new(11, 4, 10, 5, nil, "C5")
local chamber6_button = Button.new(21, 4, 10, 5, nil, "C6")

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

return Chamber

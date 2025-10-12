local Widget = require("ui.widget")
local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local List = require("ui.list")

local Config = require("src.config")
local CheckWindow = require("src.check_window")
local WarnWindow = require("src.warn_window")
local ProgressContrl = require("src.process_control")

local Recipe = {}

local function sub_recipe_widget(recipe_panel, x, y, label, add_item_callback, del_item_callback)
    local widget = Widget.new(x, y, 19, 11)

    local lable = Label.new(1, 1, 19, 1, label)
    widget:add_child(lable)

    local list = List.new(1, 2, 19, 7, {}, function()
    end)
    widget:add_child(list)

    local input = Input.new(1, 9, 13, 3, function(self)
        local item_and_count = self.text
        local item, count = string.match(item_and_count, "(%S+)%s+(%S+)")
        if not item or not count or not tonumber(count) then
            recipe_panel:disable_child_event()
            WarnWindow.newUI(recipe_panel, 16, 7, 40, 9, "format error. Must be `item count`!")
        else
            count = tonumber(count)
            add_item_callback(list, item, count)
        end
    end)
    widget:add_child(input)

    local inputs_del_button = Button.new(15, 9, 5, 3, function()
        local name_count = list:get_item()
        if name_count then
            local item, count = string.match(name_count, "(%S+)%s+(%S+)")
            del_item_callback(list, item)
        else
            recipe_panel:disable_child_event()
            WarnWindow.newUI(recipe_panel, 16, 7, 40, 9, "Must choose a recipe!")
        end
    end, "Del")

    widget:add_child(inputs_del_button)
    return widget, list
end

local function check_reicipe_selected(recipe_panel, current_recipe_name)
    if current_recipe_name then
        return true
    else
        recipe_panel:disable_child_event()
        WarnWindow.newUI(recipe_panel, 16, 7, 40, 9, "Must choose a recipe to edit!!")
        return false
    end
end

function Recipe.newUI(ec_panel)
    local recipe_panel = Panel.new(6, 2, 70, 24, "Recipe")
    recipe_panel:set_id("recipe_panel")

    local config = Config.load()
    local current_recipe_name = nil

    local inputs_recipe_widget, inputs_list = sub_recipe_widget(recipe_panel, 30, 2, "Inputs",
        function(list, item_name, item_count)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                if config:check_in_input_item(item_name) then
                    config:set_recipe_input_counts(current_recipe_name, item_name, item_count)
                    list.items(config:get_recipe_input_counts(current_recipe_name))
                else
                    recipe_panel:disable_child_event()
                    WarnWindow.newUI(recipe_panel, 2, 7, 68, 9, item_name .. " must contain chamber items.")
                end
            end
        end, function(list, item_name)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_input_counts(current_recipe_name, item_name, nil)
                list.items(config:get_recipe_input_counts(current_recipe_name))
            end
        end)
    recipe_panel:add_child(inputs_recipe_widget)

    local outputs_recipe_widget, outputs_list = sub_recipe_widget(recipe_panel, 50, 2, "Outputs",
        function(list, item_name, item_count)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_output_counts(current_recipe_name, item_name, item_count)
                list.items(config:get_recipe_output_counts(current_recipe_name))
            end
        end, function(list, item_name)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_output_counts(current_recipe_name, item_name, nil)
                list.items(config:get_recipe_output_counts(current_recipe_name))
            end
        end)
    recipe_panel:add_child(outputs_recipe_widget)

    local inputbus_recipe_widget, inputbus_list = sub_recipe_widget(recipe_panel, 30, 13, "InputBus",
        function(list, item_name, item_count)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_inputbus_counts(current_recipe_name, item_name, item_count)
                list.items(config:get_recipe_inputbus_counts(current_recipe_name))
            end
        end, function(list, item_name)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_inputbus_counts(current_recipe_name, item_name, nil)
                list.items(config:get_recipe_inputbus_counts(current_recipe_name))
            end
        end)
    recipe_panel:add_child(inputbus_recipe_widget)

    local outputbus_recipe_widget, outpubus_list = sub_recipe_widget(recipe_panel, 50, 13, "OutputBus",
        function(list, item_name, item_count)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_outputbus_counts(current_recipe_name, item_name, item_count)
                list.items(config:get_recipe_outputbus_counts(current_recipe_name))
            end
        end, function(list, item_name)
            if check_reicipe_selected(recipe_panel, current_recipe_name) then
                config:set_recipe_outputbus_counts(current_recipe_name, item_name, item_count)
                list.items(config:get_recipe_outputbus_counts(current_recipe_name))
            end
        end)
    recipe_panel:add_child(outputbus_recipe_widget)

    ----- Recipe --------------------------------------------------------------------------------
    local recipe_list_label = Label.new(3, 2, 25, 1, "Reicpes")
    recipe_panel:add_child(recipe_list_label)

    -- Recipe List
    local recipe_list = List.new(3, 3, 25, 18, config:get_recipe_names(), function(recipe_name)
        inputs_list.items(config:get_recipe_input_counts(recipe_name))
        outputs_list.items(config:get_recipe_output_counts(recipe_name))
        inputbus_list.items(config:get_recipe_inputbus_counts(recipe_name))
        outpubus_list.items(config:get_recipe_outputbus_counts(recipe_name))

        current_recipe_name = recipe_name
    end)
    recipe_panel:add_child(recipe_list)

    -- Del_Button
    local del_button = Button.new(23, 21, 5, 3, function()
        local recipe_name = recipe_list:get_item()
        if not recipe_name then
            return
        end
        recipe_panel:disable_child_event()
        CheckWindow.newUI(recipe_panel, 21, 8, 31, 7, function()
            config:del_recipes(recipe_name)
            recipe_list.items(config:get_recipe_names())
        end)
    end, "Del")
    recipe_panel:add_child(del_button)

    -- Input
    local recipe_input = Input.new(9, 21, 14, 3, function(input)
        config:add_recipes(input.text)
        recipe_list.items(config:get_recipe_names())
    end)
    recipe_panel:add_child(recipe_input)

    local quit_button = Button.new(3, 21, 6, 3, function()
        local result, msg = config:save()
        if not result then
            recipe_panel:disable_child_event()
            WarnWindow.newUI(recipe_panel, 2, 7, 68, 9, msg)
        else
            ProgressContrl.realod_config()
            ec_panel:del_child("recipe_panel")
            ec_panel:enable_child_event()
        end
    end, "Quit", 0xFFFFFF, 0xFF0000)
    recipe_panel:add_child(quit_button)

    ec_panel:add_child(recipe_panel)
end

return Recipe

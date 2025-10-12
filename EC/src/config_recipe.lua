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
local unicode = require("unicode")

local Recipe = {}

local function split(text)
    local pattern = string.format("([^%s]+)", " ")
    if not text or text == "" then
        return "", ""
    end

    local cleaned = text:gsub("\0", "")
    while unicode.sub(cleaned, 1, 1) == " " do
        cleaned = unicode.sub(cleaned, 2)
    end
    -- 清理结尾空格
    while unicode.sub(cleaned, -1) == " " do
        cleaned = unicode.sub(cleaned, 1, -2)
    end

    -- 查找第一个空格
    local first_space_pos
    for i = 1, unicode.len(cleaned) do
        if unicode.sub(cleaned, i, i) == " " then
            first_space_pos = i
            break
        end
    end

    if not first_space_pos then
        return cleaned, ""
    end

    -- 提取第一部分
    local part1 = unicode.sub(cleaned, 1, first_space_pos - 1)

    -- 提取第二部分（跳过连续空格）
    local part2 = unicode.sub(cleaned, first_space_pos + 1)
    while unicode.sub(part2, 1, 1) == " " do
        part2 = unicode.sub(part2, 2)
    end

    return part1, part2
end

local function split_by_ellipsis_unicode(text)
    if not text or text == "" then
        return "", ""
    end

    local ellipsis_start = nil
    local text_len = unicode.len(text)

    for i = 1, text_len - 2 do
        if unicode.sub(text, i, i + 2) == "..." then
            ellipsis_start = i
            break
        end
    end

    if not ellipsis_start then
        local cleaned = text
        while unicode.len(cleaned) > 0 and unicode.sub(cleaned, 1, 1) == " " do
            cleaned = unicode.sub(cleaned, 2)
        end
        while unicode.len(cleaned) > 0 and unicode.sub(cleaned, -1) == " " do
            cleaned = unicode.sub(cleaned, 1, -2)
        end
        return cleaned, ""
    end

    local part1 = unicode.sub(text, 1, ellipsis_start - 1)
    while unicode.len(part1) > 0 and unicode.sub(part1, -1) == " " do
        part1 = unicode.sub(part1, 1, -2)
    end

    local part2 = unicode.sub(text, ellipsis_start + 3)
    while unicode.len(part2) > 0 and unicode.sub(part2, 1, 1) == " " do
        part2 = unicode.sub(part2, 2)
    end

    return part1, part2
end

local function sub_recipe_widget(recipe_panel, x, y, label, add_item_callback, del_item_callback)
    local widget = Widget.new(x, y, 33, 11)

    local lable = Label.new(1, 1, 33, 1, label)
    widget:add_child(lable)

    local list = List.new(1, 2, 33, 7, {}, function()
    end)
    widget:add_child(list)

    local input = Input.new(1, 9, 27, 3, function(self)
        local item_and_count = self.text
        local item, count = split(item_and_count)
        if not item or not count or not tonumber(count) then
            recipe_panel:disable_child_event()
            WarnWindow.newUI(recipe_panel, 16, 7, 40, 9, "format error. Must be `item count`!")
        else
            count = tonumber(count)
            add_item_callback(list, item, count)
        end
    end)
    widget:add_child(input)

    local inputs_del_button = Button.new(29, 9, 5, 3, function()
        local name_count = list:get_item()
        if name_count then
            local item, count = split_by_ellipsis_unicode(name_count)
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

    local inputs_recipe_widget, inputs_list = sub_recipe_widget(recipe_panel, 35, 2, "Inputs",
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

    -- local outputs_recipe_widget, outputs_list = sub_recipe_widget(recipe_panel, 50, 2, "Outputs",
    --     function(list, item_name, item_count)
    --         if check_reicipe_selected(recipe_panel, current_recipe_name) then
    --             config:set_recipe_output_counts(current_recipe_name, item_name, item_count)
    --             list.items(config:get_recipe_output_counts(current_recipe_name))
    --         end
    --     end, function(list, item_name)
    --         if check_reicipe_selected(recipe_panel, current_recipe_name) then
    --             config:set_recipe_output_counts(current_recipe_name, item_name, nil)
    --             list.items(config:get_recipe_output_counts(current_recipe_name))
    --         end
    --     end)
    -- recipe_panel:add_child(outputs_recipe_widget)

    local inputbus_recipe_widget, inputbus_list = sub_recipe_widget(recipe_panel, 35, 13, "InputBus",
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

    -- local outputbus_recipe_widget, outpubus_list = sub_recipe_widget(recipe_panel, 50, 13, "OutputBus",
    --     function(list, item_name, item_count)
    --         if check_reicipe_selected(recipe_panel, current_recipe_name) then
    --             config:set_recipe_outputbus_counts(current_recipe_name, item_name, item_count)
    --             list.items(config:get_recipe_outputbus_counts(current_recipe_name))
    --         end
    --     end, function(list, item_name)
    --         if check_reicipe_selected(recipe_panel, current_recipe_name) then
    --             config:set_recipe_outputbus_counts(current_recipe_name, item_name, item_count)
    --             list.items(config:get_recipe_outputbus_counts(current_recipe_name))
    --         end
    --     end)
    -- recipe_panel:add_child(outputbus_recipe_widget)

    ----- Recipe --------------------------------------------------------------------------------
    local recipe_list_label = Label.new(3, 2, 30, 1, "Reicpes")
    recipe_panel:add_child(recipe_list_label)

    -- Recipe List
    local recipe_list = List.new(3, 3, 30, 18, config:get_recipe_names(), function(recipe_name)
        inputs_list.items(config:get_recipe_input_counts(recipe_name))
        -- outputs_list.items(config:get_recipe_output_counts(recipe_name))
        inputbus_list.items(config:get_recipe_inputbus_counts(recipe_name))
        -- outpubus_list.items(config:get_recipe_outputbus_counts(recipe_name))

        current_recipe_name = recipe_name
    end)
    recipe_panel:add_child(recipe_list)

    -- Del_Button
    local del_button = Button.new(28, 21, 5, 3, function()
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
    local recipe_input = Input.new(9, 21, 19, 3, function(input)
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

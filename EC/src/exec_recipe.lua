local Widget = require("ui.widget")
local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local List = require("ui.list")

local Config = require("src.config")
local CheckWindow = require("src.check_window")
local ProgressContrl = require("src.process_control")

local ExecRecipe = {}

---@class Tasks
local Tasks = {}
Tasks.__index = Tasks

---@return Tasks
function Tasks.new()
    return setmetatable({
        data = {}
    }, Tasks)
end

function Tasks:add(item_name, item_count)
    if self.data[item_name] then
        self.data[item_name] = self.data[item_name] + item_count
    else
        self.data[item_name] = item_count
    end
end

function Tasks:del(item_name)
    if self.data[item_name] then
        self.data[item_name] = nil
    end
end

function Tasks:get_item_count_list()
    local result = {}
    for k, v in pairs(self.data) do
        table.insert(result, k .. "..." .. tostring(v))
    end
    return result
end

function Tasks:submit_progress_control()
    for k, v in pairs(self.data) do
        ProgressContrl.add_task(k, v)
    end
end

function ExecRecipe.newUI(ec_panel)
    local panel = Panel.new(6, 2, 70, 24, "ExecRecipe")
    panel:set_id("exec_recipe")

    local config = Config.load()
    local tasks = Tasks.new()

    local recipe_list_label = Label.new(3, 2, 25, 1, "Recipe")
    panel:add_child(recipe_list_label)
    local recipe_list = List.new(3, 3, 25, 21, config:get_recipe_names(), function(item_name)
    end)
    panel:add_child(recipe_list)

    local task_list_label = Label.new(44, 2, 25, 1, "Tasks")
    panel:add_child(task_list_label)
    local tasks_list = List.new(44, 3, 25, 21, tasks:get_item_count_list(), function(item_name)
    end)
    panel:add_child(tasks_list)

    local count_input_label = Label.new(29, 6, 14, 1, "Count")
    panel:add_child(count_input_label)
    local count_input = Input.new(29, 7, 14, 3, function(self)
    end)
    panel:add_child(count_input)

    local add_button = Button.new(29, 10, 14, 3, function()
        local recipe_name = recipe_list:get_item()
        local count = count_input.text
        if not recipe_name or not count then
            return
        end
        count = tonumber(count)
        if not count then
            return
        end

        tasks:add(recipe_name, count)
        tasks_list.items(tasks:get_item_count_list())
    end, "Add→")
    panel:add_child(add_button)

    local del_button = Button.new(29, 13, 14, 3, function()
        local task_name_count = tasks_list:get_item()
        if not task_name_count then
            return
        end
        tasks:del(string.match(task_name_count, "([%w_]+)"))
        tasks_list.items(tasks:get_item_count_list())
    end, "Del←")
    panel:add_child(del_button)

    local confirm_button = Button.new(29, 16, 14, 3, function()
        if #tasks:get_item_count_list() == 0 then
            return
        end
        panel:disable_child_event()
        CheckWindow.newUI(panel, 21, 8, 31, 7, function()
            tasks:submit_progress_control()
            ec_panel:del_child("exec_recipe")
            ec_panel:enable_child_event()
        end)
    end, "Confirm", 0xFFFFFF, 0xFF0000)
    panel:add_child(confirm_button)

    local quit_button = Button.new(29, 21, 14, 3, function()
        ec_panel:del_child("exec_recipe")
        ec_panel:enable_child_event()
    end, "Quit")
    panel:add_child(quit_button)

    ec_panel:add_child(panel)
end

return ExecRecipe

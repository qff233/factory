local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local ProgressBar = require("ui.progressbar")
local List = require("ui.list")

local component = require("component")
local GT = component.gt_machine
local ProcessControl = require("src.process_control")

local State = {}

local status_panel = Panel.new(41, 5, 40, 21, "机器状态")

local current_progress_bar = ProgressBar.new(12, 18, 27, 3, 0.0, "current_progress_bar")
status_panel:add_child(current_progress_bar)
local current_progress_label = Label.new(3, 19, 8, 1, "current:")
status_panel:add_child(current_progress_label)

local machine_active_label = Label.new(3, 3, 36, 1, "machine_active_label", 0xFFFFFF, 0x000000)
status_panel:add_child(machine_active_label)

local current_porblem_label = Label.new(3, 5, 36, 1, "current_porblem_label", 0xFFFFFF, 0x000000)
status_panel:add_child(current_porblem_label)

local input_voltage_label = Label.new(3, 6, 36, 1, "input voltage_label", 0xFFFFFF, 0x000000)
status_panel:add_child(input_voltage_label)

local input_eu_label = Label.new(3, 7, 36, 1, "input_eu_label", 0xFFFFFF, 0x000000)
status_panel:add_child(input_eu_label)

local input_electric_label = Label.new(3, 8, 36, 1, "input_electric", 0xFFFFFF, 0x000000)
status_panel:add_child(input_electric_label)

local current_state_label = Label.new(3, 9, 36, 1, "current_state", 0xFFFFFF, 0x000000)
status_panel:add_child(current_state_label)

local task_count_label = Label.new(3, 11, 36, 1, "task_count", 0xFFFFFF, 0x000000)
status_panel:add_child(task_count_label)

local current_recipe_label = Label.new(3, 12, 36, 1, "current_recipe", 0xFFFFFF, 0x000000)
status_panel:add_child(current_recipe_label)

local remain_input_label = Label.new(3, 13, 36, 1, "remain_input", 0xFFFFFF, 0x000000)
status_panel:add_child(remain_input_label)

local remain_inputbus_label = Label.new(3, 14, 36, 1, "remain_inputbus", 0xFFFFFF, 0x000000)
status_panel:add_child(remain_inputbus_label)

function State.newUI(ec_panel)
    ec_panel:add_child(status_panel)
end

local function get_current_problem_count()
    return tonumber(GT.getSensorInformation()[5]:match("§c(%d+)"))
end

function State.update()
    if GT.hasWork() then
        machine_active_label.text("~~运行中~~")
        machine_active_label.text_color(0x00FF00)
    else
        machine_active_label.text("空闲")
        machine_active_label.text_color(0xFFFF00)
    end

    local current_problem = get_current_problem_count()
    if current_problem then
        if current_problem == 0 then
            current_porblem_label.text(" ")
            current_porblem_label.background_color(0x000000)
        else
            current_porblem_label.text("请检修机器!!")
            current_porblem_label.background_color(0xFF0000)
        end
    end

    input_voltage_label.text("当前电压：" .. tostring(GT.getInputVoltage()))

    local input_eu = "平均EU输入：" .. tostring(GT.getEUInputAverage())
    input_eu_label.text(input_eu)

    local input_electric = "平均电子输入：" .. tostring(GT.getAverageElectricInput())
    input_electric_label.text(input_electric)

    current_state_label.text(ProcessControl.get_current_state())

    local task_count = task_count_label.text(ProcessControl.get_task_count())
    if task_count > 0 then
        task_count_label.text("剩余任务数量：" .. tostring(task_count))
        task_count_label.background_color(0x00FF00)
    else
        task_count_label.text(" ")
        task_count_label.background_color(0x000000)
    end

    local recipe_name, input_count, inputbus_count = ProcessControl.get_current_task()
    if recipe_name then
        current_recipe_label.text(recipe_name)
        current_recipe_label.background_color(0x00FF00)
        remain_input_label.text("当前待输入流体量：" .. tostring(input_count))
        remain_input_label.background_color(0x00FF00)
        remain_inputbus_label.text("当前待输入物品量：" .. tostring(inputbus_count))
        remain_inputbus_label.background_color(0x00FF00)
    else
        current_recipe_label.text(" ")
        current_recipe_label.background_color(0x000000)
        remain_input_label.text(" ")
        remain_input_label.background_color(0x000000)
        remain_inputbus_label.text(" ")
        remain_inputbus_label.background_color(0x000000)
    end

    local progress = 0
    if GT.getWorkMaxProgress() ~= 0 then
        progress = GT.getWorkProgress() / GT.getWorkMaxProgress()
    end
    current_progress_bar.value(progress)

end

return State

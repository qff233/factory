local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local ProgressBar = require("ui.progressbar")
local List = require("ui.list")

local ConfigRecipe = require("src.config_recipe")
local ExecRecipe = require("src.exec_recipe")
local State = require("src.state")
local Chamber = require("src.chamber")

local component = require("component")
local GT = component.gt_machine
local Computer = require("computer")
local ProcessControl = require("src.process_control")

local ec_panel = Panel.new(1, 1, 80, 25, "EC")

local function power_button()
    local button_color
    if GT.isWorkAllowed() then
        button_color = 0x00FF00
    else
        button_color = 0xFF0000
    end
    local power_button = Button.new(74, 2, 6, 3, nil, "⏼", 0xFFFFFF, button_color)
    local function on_power_button_clicked()
        if GT.isWorkAllowed() then
            GT.setWorkAllowed(false)
        else
            GT.setWorkAllowed(true)
        end
    end
    power_button.on_clicked = on_power_button_clicked
    return power_button
end

local function state_button()
    local state_button = Button.new(56, 2, 15, 3, function()
    end, "OnlineRemote", 0xFFFFFF, 0x00FF00)
    return state_button
end

local function recipe_button()
    local recipe_button = Button.new(2, 2, 15, 3, function()
        ec_panel:disable_child_event()
        ConfigRecipe.newUI(ec_panel)
    end, "配置配方")
    return recipe_button
end

local function exec_button()
    local exec_button = Button.new(20, 2, 15, 3, function()
        ec_panel:disable_child_event()
        ExecRecipe.newUI(ec_panel)
    end, "执行配方")
    return exec_button
end

local power_button = power_button()
ec_panel:add_child(power_button)

local state_button = state_button()
ec_panel:add_child(state_button)

local recipe_button = recipe_button()
ec_panel:add_child(recipe_button)

local exec_button = exec_button()
ec_panel:add_child(exec_button)

local status_panel = State.newUI(ec_panel)
local chamber_panel = Chamber.newUI(ec_panel)

local computer_ram_state_progress = ProgressBar.new(37, 2, 17, 3, 1.0)
computer_ram_state_progress.border_color = 0xFFFFFF
ec_panel:add_child(computer_ram_state_progress)

local chamber_buttons = Chamber.getChamberButton()

---@return Widget
function Main()
    ProcessControl.realod_config()
    return ec_panel
end

function Update()
    for k, v in pairs(chamber_buttons) do
        v.background_color(0x00FF00)
    end
    ProcessControl.update(function(chamber_id)
        chamber_buttons[chamber_id].background_color(0xFFFF00)
    end)
    State.update()

    local ram_percent = (Computer.totalMemory() - Computer.freeMemory()) / Computer.totalMemory()
    if ram_percent > 0.9 then
        computer_ram_state_progress.bar_color(0xFF0000)
    elseif ram_percent > 0.6 then
        computer_ram_state_progress.bar_color(0xFFFF00)
    else
        computer_ram_state_progress.bar_color(0x00FF00)
    end
    computer_ram_state_progress.value(ram_percent)

    if GT.isWorkAllowed() then
        power_button.background_color(0x00FF00)
    else
        power_button.background_color(0xFF0000)
    end
end

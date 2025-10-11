local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local ProgressBar = require("ui.progressbar")
local List = require("ui.list")

local Recipe = require("src.recipe")
local State = require("src.state")
local Chamber = require("src.chamber")

local component = require("component")
local GT = component.gt_machine
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
        local recipe_panel = Recipe.newUI(ec_panel)
        ec_panel:add_child(recipe_panel)
    end, "Recipe")
    return recipe_button
end

local function exec_button()
    local exec_button = Button.new(33, 2, 15, 3, function()
    end, "Exec")
    return exec_button
end

local power_button = power_button()
local state_button = state_button()
local recipe_button = recipe_button()
local exec_button = exec_button()
ec_panel:add_child(power_button)
ec_panel:add_child(state_button)
ec_panel:add_child(recipe_button)
ec_panel:add_child(exec_button)

local status_panel = State.newUI(ec_panel)
local chamber_panel = Chamber.newUI(ec_panel)

---@return Widget
function Main()
    ProcessControl.realod_config()
    ProcessControl.add_task("N1H1", 50)
    return ec_panel
end

function Update()
    State.update()

    ProcessControl.update()

    if GT.isWorkAllowed() then
        power_button.background_color(0x00FF00)
    else
        power_button.background_color(0xFF0000)
    end
end

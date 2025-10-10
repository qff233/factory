local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")
local ProgressBar = require("ui.progressbar")
local List = require("ui.list")

function Recipe(ec_panel)
    local recipe_panel = Panel.new(6, 2, 70, 23, "Recipe")
    recipe_panel:set_id("recipe_panel")
    local ok_button = Button.new(65, 20, 4, 3, function()
        ec_panel:del_child("recipe_panel")
        ec_panel:enable_child_event()
    end, "OK")
    local add_button = Button.new(59, 20, 5, 3, function()
    end, "Add")

    recipe_panel:add_child(ok_button)
    recipe_panel:add_child(add_button)

    return recipe_panel
end

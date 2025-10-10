local Event = require("event")
local Computer = require("computer")

local Label = require("ui.label")
local Panel = require("ui.panel")
local Button = require("ui.button")
local Input = require("ui.input")
local CheckBox = require("ui.checkbox")

local panel = Panel.new(1, 1, 80, 25, "EC")
local label = Label.new(41, 20, 20, 1, "1", 0x666666, 0x999999)
local button = Button.new(30, 10, 10, 10, function()
    print("clicked~")
end, "123456")
local input = Input.new(10, 10, 20, 3, function(value)
    label.text(value)
end)
local checkbox = CheckBox.new(2, 2, function(value)
    if value then
        label:enable_visible()
    else
        label:disable_visible()
    end
end, true, "tests")

panel:add_child(label)
panel:add_child(button)
panel:add_child(input)
panel:add_child(checkbox)
panel:draw()

while true do
    local event = {Event.pull(1)}
    if event then
        panel:handle_evnet(event)
        panel:draw()
    end
    -- TODO
end

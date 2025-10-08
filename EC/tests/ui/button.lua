local Widget = require("ui.widget")
local Button = require("ui.button")

local widget = Widget.new(1, 1, 100, 100)
local button = Button.new(1, 1, 10, 10, function()
    print("clicked~")
end)

widget:add_child(button)
widget:draw()

local events = {{"touch", nil, 50, 50}, {"touch", nil, 5, 5}, {"drag", nil, 50, 50}, {"drop", nil, 50, 50}}

for _, event in ipairs(events) do
    print(event[1])
    widget:handle_evnet(event)
    widget:draw()
end

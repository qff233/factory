local Widget = require("ui.widget")
local Input = require("ui.input")

local widget = Widget.new(1, 1, 100, 100)
local input = Input.new(1, 1, 10, 10, function(str)
    print("submit a string:", str)
end)

widget:add_child(input)
widget:draw()

local events = {{"touch", nil, 50, 50}, {"touch", nil, 5, 5}, {"key_down", nil, 'h', 0xFF},
                {"key_down", nil, 'e', 0xFF}, {"key_down", nil, 'l', 0xFF}, {"key_down", nil, 'l', 0xFF},
                {"key_down", nil, 'o', 0xFF}, {"key_down", nil, '!', 0xFF}, {"key_down", nil, 'G', 0x1C}}

for _, event in ipairs(events) do
    print(event[3])
    widget:handle_evnet(event)
    widget:draw()
end

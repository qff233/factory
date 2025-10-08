local Widget = require("ui.widget")
local Label = require("ui.label")

local widget = Widget.new(1, 1, 100, 100)
local label = Label.new(1, 1, 10, 10, "12345")

widget:add_child(label)
print("draw-------------------")
widget:draw()
print("-----------------------")
print("draw-------------------")
widget:draw()
print("-----------------------")

print("change text")
label.text("123123123")

print("draw-------------------")
widget:draw()
print("-----------------------")

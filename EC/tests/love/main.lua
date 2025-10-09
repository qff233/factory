local component = require("component.init")
local gpu = component.gpu
gpu.new()
local signal_queue = {}

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

function love.load()
    panel:draw()
end

function love.update()
    if #signal_queue > 0 then
        local signal = table.remove(signal_queue, 1)
        panel:handle_evnet(signal)
        panel:draw()
    end

end

function love.draw()
    love.graphics.draw(gpu.screen, 0, 0)
end

function love.mousepressed(x, y, button, istouch, presses)
    local charX = math.floor(x / gpu.font_width) + 1
    local charY = math.floor(y / gpu.font_height) + 1

    print(charX, charY)
    local signal = {"touch", "screen_0", -- 模拟的屏幕地址
    charX, charY, button, "player"}
    table.insert(signal_queue, signal)
end

function love.mousereleased(x, y, button, istouch, presses)
    local charX = math.floor(x / gpu.font_width) + 1
    local charY = math.floor(y / gpu.font_height) + 1

    local signal = {"drop", "screen_0", -- 模拟的屏幕地址
    charX, charY, button, "player"}
    table.insert(signal_queue, signal)
end

function love.keypressed(key, scancode, isrepeat)
    local char = key
    if scancode == "return" then
        scancode = 0x1C
    elseif scancode == "left" then
        scancode = 0xCB
    elseif scancode == "right" then
        scancode = 0xCD
    elseif scancode == "backspace" then
        scancode = 0x0E
    end
    local signal = {"key_down", "keyboard_0", char, scancode, "player"}
    table.insert(signal_queue, signal)
end

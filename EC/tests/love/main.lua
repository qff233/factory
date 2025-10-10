local component = require("component")
local gpu = component.gpu
gpu.new()
local signal_queue = {}

require("src.recipe_ui")
require("src.main_ui")

local main_ui = MainUI()
function love.load()
    main_ui:draw()
end

function love.update()
    if #signal_queue > 0 then
        local signal = table.remove(signal_queue, 1)
        main_ui:handle_evnet(signal)
        main_ui:draw()
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
    local signal = {"key_down", "keyboard_0", string.byte(char), scancode, "player"}
    table.insert(signal_queue, signal)
end

function love.wheelmoved(x, y)
    local signal = {"scroll", null, 0, 0, y}
    table.insert(signal_queue, signal)
end

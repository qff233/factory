local GPU = {}

---@class GPU
---@field background number
---@field foreground number
---@field font_width number
---@field font_height number
---@field screen any
local GPU = {}

function GPU.new()
    ---@type GPU
    GPU.background = 0x000000
    GPU.foreground = 0xFFFFFF

    local font = love.graphics.newFont("font.ttf", 16)
    GPU.font_width = font:getWidth("1")
    GPU.font_height = font:getHeight()
    GPU.screen = love.graphics.newCanvas(80 * GPU.font_width, 25 * GPU.font_height)

    print(GPU.font_width, GPU.font_height)
    love.graphics.setFont(font)

    GPU.fill(1, 1, 80, 25, " ")
end

---@param color number
function GPU.setBackground(color)
    GPU.background = color
end

---@param color number
function GPU.setForeground(color)
    GPU.color = color
end

---@param x number
---@param y number
---@param width number
---@param height number
---@param char string
function GPU.fill(x, y, width, height, char)
    local bg_r = bit.band(bit.rshift(GPU.background, 16), 0xFF) / 255
    local bg_g = bit.band(bit.rshift(GPU.background, 8), 0xFF) / 255
    local bg_b = bit.band(GPU.background, 0xFF) / 255

    local fg_r = bit.band(bit.rshift(GPU.foreground, 16), 0xFF) / 255
    local fg_g = bit.band(bit.rshift(GPU.foreground, 8), 0xFF) / 255
    local fg_b = bit.band(GPU.foreground, 0xFF) / 255

    local font_width = GPU.font_width
    local font_height = GPU.font_height
    local pixel_x = (x - 1) * font_width
    local pixel_y = (y - 1) * font_height
    local pixel_width = width * font_width
    local pixel_height = height * font_height

    love.graphics.setCanvas(GPU.screen)
    love.graphics.setColor(bg_r, bg_g, bg_b)
    love.graphics.rectangle("fill", pixel_x, pixel_y, pixel_width, pixel_height)

    if char and char ~= " " and char ~= "" then
        love.graphics.setColor(fg_r, fg_g, fg_b)

        for i = 0, width - 1 do
            for j = 0, height - 1 do
                local char_x = pixel_x + i * font_width
                local char_y = pixel_y + j * font_height
                love.graphics.print(char, char_x, char_y)
            end
        end
    end
    love.graphics.setCanvas()
end

local function utf8len(input)
    local len = #input
    local left = len
    local cnt = 0
    local arr = {0, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc}
    while left ~= 0 do
        local tmp = string.byte(input, -left)
        local i = #arr
        while arr[i] do
            if tmp >= arr[i] then
                left = left - i
                break
            end
            i = i - 1
        end
        cnt = cnt + 1
    end
    return cnt
end

---@param x number
---@param y number
---@param value number
---@param vertical? boolean
function GPU.set(x, y, value, vertical)
    local bg_r = bit.band(bit.rshift(GPU.background, 16), 0xFF) / 255
    local bg_g = bit.band(bit.rshift(GPU.background, 8), 0xFF) / 255
    local bg_b = bit.band(GPU.background, 0xFF) / 255

    local fg_r = bit.band(bit.rshift(GPU.foreground, 16), 0xFF) / 255
    local fg_g = bit.band(bit.rshift(GPU.foreground, 8), 0xFF) / 255
    local fg_b = bit.band(GPU.foreground, 0xFF) / 255

    local font_width = GPU.font_width
    local font_height = GPU.font_height
    local pixel_x = (x - 1) * font_width
    local pixel_y = (y - 1) * font_height

    local text_width = utf8len(value) * font_width
    love.graphics.setCanvas(GPU.screen)
    love.graphics.setColor(bg_r, bg_g, bg_b)
    love.graphics.rectangle("fill", pixel_x, pixel_y, text_width, font_height)

    love.graphics.setColor(fg_r, fg_g, fg_b)
    if vertical then
        for i = 1, utf8len(value) do
            local char = value:sub(i, i)
            love.graphics.print(char, pixel_x, pixel_y + (i - 1) * font_height)
        end
    else
        love.graphics.print(value, pixel_x, pixel_y)
    end
    love.graphics.setCanvas()
end

local internet = {}

---@param url string
---@param data table
---@param headers table
---@param method string
function internet.request(url, data, headers, method)
    print("request")
end

local GT = {
    work_allow = false
}

---@return number
function GT.getWorkMaxProgress()
    return 100
end

function GT.getSensorInformation()
    return {"", "", "", "", "问题：§c6§r 效率：§e0.0§r %", ""}
end

---@return number
function GT.getWorkProgress()
    return 50
end

---@return void
function GT.setWorkAllowed(value)
    GT.work_allow = value
end

---@return boolean
function GT.isWorkAllowed()
    return GT.work_allow
end

function GT.getInputVoltage()
    return 6666
end

function GT.hasWork()
    return true
end

function GT.getEUInputAverage()
    return 999
end

function GT.getAverageElectricInput()
    return 888
end

---@class Transposer
local Transposer = {}

---@field sourceSide number
---@field sinkSide number
---@field count number
---@field sourceTank number
---@return boolean, number
function Transposer.transferFluid(sourceSide, sinkSide, count, sourceTank)
    return true, count
end

---@field sourceSide number
---@field sinkSide number
---@field count number
---@field sourceSlot number
---@return number
function Transposer.transferItem(sourceSide, sinkSide, count, sourceSlot)
    return count
end

function Transposer.getFluidInTank(sourceSide)
    return {{
        lable = "氢气"
    }, {
        lable = "氮气"
    }}
end

function Transposer.getAllStacks(sourceSide)
    local items = {{
        lable = "氢氧化钠粉"
    }}
    local key, current = next(items)
    return function()
        key, current = next(items, key)
        return current
    end
end

local component = {
    gpu = GPU,
    internet = internet,
    gt_machine = GT
}

---@return Transposer
function component.proxy()
    return Transposer

end

return component

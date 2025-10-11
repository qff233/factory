local M = {}

---@param codepoint number
---@return string
function M.char(codepoint)
    if codepoint <= 0x7F then
        return string.char(codepoint)
    elseif codepoint <= 0x7FF then
        return string.char(0xC0 + math.floor(codepoint / 0x40), 0x80 + (codepoint % 0x40))
    elseif codepoint <= 0xFFFF then
        return string.char(0xE0 + math.floor(codepoint / 0x1000), 0x80 + (math.floor(codepoint / 0x40) % 0x40),
            0x80 + (codepoint % 0x40))
    else
        return string.char(0xF0 + math.floor(codepoint / 0x40000), 0x80 + (math.floor(codepoint / 0x1000) % 0x40),
            0x80 + (math.floor(codepoint / 0x40) % 0x40), 0x80 + (codepoint % 0x40))
    end
end

function M.len(input)
    local lenInByte = #input
    local width = 0
    local i = 1
    while (i <= lenInByte) do
        local curByte = string.byte(input, i)
        local byteCount = 1;
        if curByte > 0 and curByte <= 127 then
            byteCount = 1 -- 1字节字符
        elseif curByte >= 192 and curByte < 223 then
            byteCount = 2 -- 双字节字符
        elseif curByte >= 224 and curByte < 239 then
            byteCount = 3 -- 汉字
        elseif curByte >= 240 and curByte <= 247 then
            byteCount = 4 -- 4字节字符
        end

        local char = string.sub(input, i, i + byteCount - 1)

        i = i + byteCount -- 重置下一字节的索引
        width = width + 1 -- 字符的个数（长度）
    end
    return width
end

function M.sub(str, b, e)
    return string.sub(str, b, e)
end

function M.charWidth(char)
    return 1
end

return M

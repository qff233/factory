local json = require("json")
local component = require("component")
local internet = component.internet

local jsonrpc = {}

function jsonrpc.proxy(url)
    return setmetatable({}, {
        __index = function(self, method)
            return function(...)
                return jsonrpc.call(url, method, ...)
            end
        end
    })
end

local function utf8len(input)
    if not input then
        return 0
    end
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

function jsonrpc.call(url, method, ...)
    local json_requrest = json.encode({
        id = tostring(math.random()),
        method = method,
        jsonrpc = "2.0",
        params = ...
    })

    local headers = {
        ["content-type"] = "application/json-rpc",
        ["content-length"] = utf8len(json_requrest)
    }

    print(utf8len(json_requrest))
    local handle = internet.request(url, json_requrest, headers, "POST")
    local result = ""
    if handle then
        for chunk in handle do
            result = result .. chunk
        end
    else
        return nil, "internet handle error"
    end

    local code = handle.response()
    if code ~= 200 then
        return nil, "HTTP ERROR: " .. code
    end

    result = json.decode(result)
    if result.result then
        return result.resut
    else
        return nil, result.error
    end
end

local jsontest = jsonrpc.proxy("http://0.0.0.0:5000")
local param = json.encode({
    id = 2000,
    position = {0, 0, 0},
    battery_level = 1.0,
    tool_level = 1.0
})
local result, msg = jsontest.vehicle_get_action(param)
print(result, msg)

return jsonrpc

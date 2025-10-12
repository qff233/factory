local M = {}

---@class Modem
---@field handles [{remote_address: string, port: number},fun(message: string)][]
Modem = {}
Modem.__index = Modem

---comment
---@return Modem
function M.get_modem()
    local self = setmetatable({}, Modem)
    self.handles = {}
    return self
end

---comment
---@param remote_address string
---@param port number
---@param callback fun(message: string)
function Modem:add_handles(remote_address, port, callback)
    table.insert(self.handles, { { remote_address = remote_address, port = port }, callback })
end

---comment
function Modem:recv()
    local _, _, remote_address, port, _, message -- = event.pull("modem_message")
    for _, handle in ipairs(self.handles) do
        local handle_remote_address, handle_port = table.unpack(handle[1])
        if remote_address == handle_remote_address and port == handle_port then
            handle[2](message)
        end
    end
end

return M

local modem = require("modem").get_modem()
local OHT = require("oht").new()
local config = require("config")

---comment
---@param message string
local function handle(message)

end

function Main()
    modem:add_handles(config.remote_address, config.port, handle)
    while true do
        modem:recv()
    end
end

Main()

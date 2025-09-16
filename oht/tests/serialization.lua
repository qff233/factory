local serialization = require("serialization")

local commands = {
    { move = { 0, 1, 2.4 } },
    { suck = 0 },
    { drop = 1 },
    { drain = 2 },
    { fill = 3 }
}

local data_packet = serialization.serialie(commands)
print(data_packet)

commands = serialization.deserialie(data_packet)
for _, command in ipairs(commands) do
    for command_id, param in pairs(command) do
        if command_id == "move" then
            local x, y, z = table.unpack(param)
            print(command_id, x, y, z)
        else
            print(command_id, param)
        end
    end
end

local M = {}

---@alias MoveCommand {move: [number,number,number]}
---@alias SuckCommand {suck: Sides}
---@alias DropCommand {drop: Sides}
---@alias DrainCommand {drain: Sides}
---@alias FillCommand {fill: Sides}
---@alias Command MoveCommand
---| SuckCommand
---| DropCommand
---| DrainCommand
---| FillCommand

---@param commands Command[]
---@return string
function M.serialie(commands)
    ---@type string
    local data_packet = ""

    for _, command in pairs(commands) do
        local command_id
        if command.move then
            command_id = "move"
        elseif command.suck then
            command_id = "suck"
        elseif command.drop then
            command_id = "drop"
        elseif command.drain then
            command_id = "drain"
        elseif command.fill then
            command_id = "fill"
        end

        local data
        if command_id == "move" then
            local x, y, z = table.unpack(command.move)
            data = string.format("%s,%s,%s,%s\n", command_id, x, y, z)
        else
            data = string.format("%s,%s\n", command_id, command[command_id])
        end

        data_packet = data_packet .. data
    end

    return data_packet
end

---@param data_packet string
---@return Command[]
function M.deserialie(data_packet)
    local lines = {}
    for line in string.gmatch(data_packet, "[^\n]+\n") do
        table.insert(lines, line)
    end

    local commands = {};
    for _, line in ipairs(lines) do
        if string.find(line, "move") then
            local x, y, z = string.match(line, "move,([%w.]+),([%w.]+),([%w.]+)\n")
            table.insert(commands, { move = { x, y, z } })
        else
            local command_id, side = string.match(line, "(%w+),(%w+)\n")
            local command = {}
            command[command_id] = side
            table.insert(commands, command)
        end
    end

    return commands
end

return M

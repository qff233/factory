---@module "command_parse"
local command_parse = require("command_parse")

-- {stone,item,[0,0,0],[1,1,1]}
local data_packet = command_parse.serialie(
  { {
    item = "stone",
    type = "item",
    from = "S",
    to = "CR1"
  },
    {
      item = "stone",
      type = "item",
      from = "CR1",
      to = "S"
    }
  });

print(data_packet)

local commands = command_parse.deserialie(data_packet)
local command = commands[1]
print(command.item, command.type, command.from, command.to)

command = commands[2]
print(command.item, command.type, command.from, command.to)

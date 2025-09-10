local M = {}

---@alias ItemType
---| '"item"'
---| '"liquid"'

---@alias Pos {x: number, y: number, z: number}
---@alias Command {item: string, type: ItemType, from: string, to: string}
---@alias Commands Command[]

---@param commands Commands
---@return string
function M.serialie(commands)
  ---@type string
  local data_packet = ""

  for _, command in pairs(commands) do
    local item = command.item
    local type = command.type
    local from = command.from
    local to = command.to
    local data = string.format("{%s,%s,%s,%s}\n", item, type, from, to)
    data_packet = data_packet .. data
  end

  return data_packet
end

---@param data_packet string
---@return Commands
function M.deserialie(data_packet)
  local commands = {};
  -- print(data_packet)
  for item, type, from, to in string.gmatch(data_packet, "{(%w+),(%w+),(%w+),(%w+)}") do
    table.insert(commands, {
      item = item,
      type = type,
      from = from,
      to = to,
    });
  end
  return commands
end

return M

local M = {}

---@alias NodeType "station"|"machine"

---@class OHTNode
---@field node_id string
---@field position Pos
---@field type NodeType
OHTNode = {}
OHTNode.__index = OHTNode

---@param node_id string
---@param pos Pos
---@param node_type NodeType
---@return OHTNode
function OHTNode.new(node_id, pos, node_type)
  local self = setmetatable({}, OHTNode)
  self.node_id = node_id
  self.position = pos
  self.type = node_type
  return self
end

---@class OHTEdge
---@field edge_id string
---@field from_node string Node_id
---@field to_node string Node_id
---@field length number
---@field is_locked boolean
OHTEdge = {}
OHTEdge.__index = OHTEdge

---@param edge_id string
---@param from_node_id string
---@param to_node_id string
---@param length number
---@return OHTEdge
function OHTEdge.new(edge_id, from_node_id, to_node_id, length)
  local self = setmetatable({}, OHTEdge)
  self.edge_id = edge_id
  self.from_node = from_node_id
  self.to_node = to_node_id
  self.length = length
  self.is_locked = false
  return self
end

-------------------------------------------------------------------------------------------------------------------------

---comment
---@class OHTTrackgraph
---@field nodes table<string, OHTNode> key=node_id,value=OHTNode
---@field edges table<string, OHTEdge> key=edge_id,value=OHTEdge
---@field adjacency_list table<string, OHTEdge[]> key=node_id,value=array of OHTEdge
local OHTTrackGraph = {}
OHTTrackGraph.__index = OHTTrackGraph

---comment
---@return OHTTrackgraph
function M.new_graph()
  local self = setmetatable({}, OHTTrackGraph)
  self.nodes = {}
  self.edges = {}
  self.adjacency_list = {}
  return self
end

---comment
---@param node_id string
---@param pos Pos
---@param node_type NodeType
function OHTTrackGraph:add_node(node_id, pos, node_type)
  local node = OHTNode.new(node_id, pos, node_type)
  self.nodes[node_id] = node
  self.adjacency_list[node_id] = {}
end

---comment
---@param edge_id string
---@param from_node_id string
---@param to_node_id string
---@param length number
function OHTTrackGraph:add_edge(edge_id, from_node_id, to_node_id, length)
  if not self.nodes[from_node_id] then
    error("From node does not exist: " .. from_node_id)
  end
  if not self.nodes[to_node_id] then
    error("From node does not exist: " .. from_node_id)
  end

  local edge = OHTEdge.new(edge_id, from_node_id, to_node_id, length)
  self.edges[edge_id] = edge

  table.insert(self.adjacency_list[from_node_id], edge)
end

---comment
---@param node_id string
---@return OHTEdge[]
function OHTTrackGraph:get_edges_from_node(node_id)
  return self.adjacency_list[node_id] or {}
end

---comment
---@param node_id string
---@return OHTNode
function OHTTrackGraph:get_node(node_id)
  return self.nodes[node_id]
end

---comment
---@param edge_id string
---@return OHTEdge
function OHTTrackGraph:get_edge(edge_id)
  return self.edges[edge_id]
end

---comment
---@return OHTNode
function OHTTrackGraph:get_nodes()
  return self.nodes
end

---comment
---@return OHTEdge
function OHTTrackGraph:get_edges()
  return self.edges
end

function OHTTrackGraph:set_edge_lock(edge_id, is_locked)
  local edge = self.edges[edge_id]
  if edge == nil then
    return false, "Edge does not exist: " .. edge_id
  end
  edge.is_locked = is_locked
  return true
end

function OHTTrackGraph:find_shortest_path(start_node_id, end_node_id)
  if not self.nodes[start_node_id] then
    return false, "Start node does not exist: " .. start_node_id
  end
  if not self.nodes[end_node_id] then
    return false, "End node does not exist: " .. end_node_id
  end

  local distances = {}
  local previous = {}
  local unvisited = {}

  for node_id, _ in pairs(self.nodes) do
    distances[node_id] = math.huge
    previous[node_id] = nil
    unvisited[node_id] = true
  end
  distances[start_node_id] = 0

  while next(unvisited) do
    local current_node = nil
    local min_distance = math.huge

    for node_id, _ in pairs(unvisited) do
      if distances[node_id] < min_distance then
        min_distance = distances[node_id]
        current_node = node_id
      end
    end

    if not current_node or min_distance == math.huge then
      break
    end

    if current_node == end_node_id then
      local path = {}
      local node = end_node_id

      while node do
        table.insert(path, 1, node)
        node = previous[node]
      end

      return path
    end

    unvisited[current_node] = nil

    local edges = self.adjacency_list[current_node] or {}
    for _, edge in ipairs(edges) do
      if edge.is_locked then
        goto continue
      end
      local neighbor = edge.to_node
      local alt = distances[current_node] + edge.length

      if alt < distances[neighbor] then
        distances[neighbor] = alt
        previous[neighbor] = current_node
      end
        ::continue::
    end
  end
end

return M

local M = {}

local queue = require("container.queue")

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
---@field weight number
---@field is_locked boolean
OHTEdge = {}
OHTEdge.__index = OHTEdge

---@param edge_id string
---@param from_node_id string
---@param to_node_id string
---@param weight number
---@return OHTEdge
function OHTEdge.new(edge_id, from_node_id, to_node_id, weight)
    local self = setmetatable({}, OHTEdge)
    self.edge_id = edge_id
    self.from_node = from_node_id
    self.to_node = to_node_id
    self.weight = weight
    self.is_locked = false
    return self
end

---comment
---@param node_a OHTNode
---@param node_b OHTNode
---@return number
local function heuristic_distance(node_a, node_b)
    local dx = node_a.position.x - node_b.position.x
    local dy = node_a.position.y - node_b.position.y
    local dz = node_a.position.z - node_b.position.z

    return math.sqrt(dx * dx + dy * dy + dz * dz)
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
---@param weight number
function OHTTrackGraph:add_edge(edge_id, weight)
    local from_node_id, to_node_id = string.match(edge_id, "(%w+)-(%w+)")
    if not self.nodes[from_node_id] then
        error("From node does not exist: " .. from_node_id)
    end
    if not self.nodes[to_node_id] then
        error("From node does not exist: " .. from_node_id)
    end

    local edge = OHTEdge.new(edge_id, from_node_id, to_node_id, weight)
    self.edges[edge_id] = edge

    table.insert(self.adjacency_list[from_node_id], edge)
end

---@alias NodeConfig [string, [number, number, number], NodeType]
---@alias EdgeConfig [string, number]

---comment
---@param track_config {nodes: NodeConfig[], edges: EdgeConfig[]}
function OHTTrackGraph:load_config(track_config)
    local nodes = track_config.nodes
    local edges = track_config.edges

    for _, node in ipairs(nodes) do
        local node_id = node[1]
        local x, y, z = table.unpack(node[2])
        local type = node[3]

        self:add_node(node_id, { x = x, y = y, z = z }, type)
    end

    for _, edge in ipairs(edges) do
        local edge_id = edge[1]
        local weight = edge[2]

        self:add_edge(edge_id, weight)
    end
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

    local goal_node = self.nodes[end_node_id]
    local open_set = queue.new_priority_queue()
    ---@type {[string] : boolean}
    local closed_set = {}
    ---@type {[string] : string}
    local came_from = {}
    ---@type {[string] : number}
    local g_score = {}
    for node_id, _ in pairs(self.nodes) do
        g_score[node_id] = math.huge
    end
    g_score[start_node_id] = 0

    open_set:push(start_node_id, heuristic_distance(self.nodes[start_node_id], goal_node))

    while not open_set:is_empty() do
        local current_id = open_set:pop()

        if current_id == end_node_id then
            local path = {}
            local node = end_node_id
            while node do
                table.insert(path, 1, node)
                node = came_from[node]
            end

            -- for key, value in pairs(g_score) do
            --     print(key, value)
            -- end

            return path
        end

        ---@cast current_id string
        closed_set[current_id] = true
        local edges = self.adjacency_list[current_id] or {}
        for _, edge in pairs(edges) do
            ---@cast edge OHTEdge
            if edge.is_locked then
                goto continue
            end

            local neighbor_id = edge.to_node
            if closed_set[neighbor_id] then
                goto continue
            end

            local tentative_g_score = g_score[current_id] + edge.weight
            if tentative_g_score >= g_score[neighbor_id] then
                goto continue
            end

            came_from[neighbor_id] = current_id
            g_score[neighbor_id] = tentative_g_score
            local f_score = g_score[neighbor_id] + heuristic_distance(self.nodes[neighbor_id], goal_node)

            if not open_set:contains(neighbor_id) then
                open_set:push(neighbor_id, f_score)
            end

            ::continue::
        end
    end
    return false, "No path found from " .. start_node_id .. " to " .. end_node_id
end

return M

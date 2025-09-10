local oht_track = require"oht_track"

local track = oht_track.new_graph()
track:add_node("S", {x=0, y=0, z=0}, "station")
track:add_node("CR1", {x=0, y=5, z=0}, "machine")
track:add_node("CR2", {x=0, y=10, z=0}, "machine")

track:add_edge("S-CR1", "S", "CR1", 5)
track:add_edge("S-CR2", "S", "CR2", 10)
track:add_edge("CR1-CR2", "CR1", "CR2", 5)
track:add_edge("CR1-S", "CR1", "S", 5)
track:add_edge("CR2-S", "CR2", "S", 10)

print("Edges from node CR1:")
local edges = track:get_edges_from_node("CR1")
for _, edge in ipairs(edges) do
  print("  " .. edge.edge_id .. ": " .. edge.from_node .. " -> " .. edge.to_node)
end



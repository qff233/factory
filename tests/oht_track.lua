local oht_track = require "oht_track"

local track = oht_track.new_graph()
track:add_node("A", { x = 0, y = 0, z = 0 }, "station")
track:add_node("B", { x = 0, y = -3, z = 0 }, "machine")
track:add_node("C", { x = 6, y = -3, z = 0 }, "machine")
track:add_node("D", { x = 7, y = 0, z = 0 }, "machine")
track:add_node("E", { x = 18, y = 0, z = 0 }, "machine")
track:add_node("F", { x = 18, y = -4, z = 0 }, "machine")

track:add_edge("A-B", 3)
track:add_edge("A-D", 7)
track:add_edge("B-C", 6)
track:add_edge("C-D", 8)
track:add_edge("C-E", 7)
track:add_edge("C-F", 10)
track:add_edge("D-C", 8)
track:add_edge("D-E", 11)
track:add_edge("E-F", 7)

local path = track:find_shortest_path("A", "F")
if type(path) == "table" then
    for _, value in pairs(path) do
        print(value .. "->")
    end
end

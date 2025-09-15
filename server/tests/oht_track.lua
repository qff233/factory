local oht_track = require("oht_control.track")
local oht_config = require("oht_track_config")

local track = oht_track.new_graph()
track:load_config(oht_config)

local path = track:find_shortest_path("A", "F")
if type(path) == "table" then
    for _, value in pairs(path) do
        print(value .. "->")
    end
end

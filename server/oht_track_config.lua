local M = {
    nodes = {
        { "A", { 0, 0, 0 },     "station" },
        { "B", { 0, 3, 0 },     "machine" },
        { "C", { 6.5, 5.5, 0 }, "machine" },
        { "D", { 3, 0, 0 },     "machine" },
        { "E", { 18, 0, 0 },    "machine" },
        { "F", { 17, 5, 0 },    "machine" },
    },
    edges = {
        { "A-B", 3 },
        { "A-D", 7 },
        { "B-C", 6 },
        { "C-D", 8 },
        { "C-E", 7 },
        { "C-F", 10 },
        { "D-C", 8 },
        { "D-E", 11 },
        { "E-F", 7 }
    }
}

return M

require("ui.widget")

local dirty = false
local a = WATCHABLE(1).set(function()
    dirty = true
end)

print(a(), dirty)
a(2)
print(a(), dirty)

local Event = require("event")
local Computer = require("computer")

require("src.main")
local main_ui = Main()

while true do
    local event = {Event.pull(0.05)}
    if event then
        main_ui:handle_evnet(event)
    end
    Update()
    main_ui:draw()
end

local Config = require("src.config")
local component = require("component")

---@class Task
---@field recipe_name string
---@field will_input table<string, number>
---@field will_inputbus table<string, number>
local Task = {}

---@class ProcessControl
---@field tasks Task[]
local ProcessControl = {}
local recipes = {}
local chamber_inputs = {}

---@type Task[]
local tasks = {}

---@type Transposer[]
local transposers = {}

local function deepcopy(orig)
    local orig_type = type(orig)
    local copy
    if orig_type == 'table' then
        copy = {}
        for orig_key, orig_value in next, orig, nil do
            copy[deepcopy(orig_key)] = deepcopy(orig_value)
        end
        setmetatable(copy, deepcopy(getmetatable(orig)))
    else
        copy = orig
    end
    return copy
end

---@param recipe_name string
function ProcessControl.add_task(recipe_name, count)
    local count = count or 1

    local recipe = recipes[recipe_name]
    for i = 1, count do
        local input = deepcopy(recipe.inputs)
        local inputbus = deepcopy(recipe.inputbus)
        table.insert(tasks, {
            recipe_name = recipe_name,
            will_input = input,
            will_inputbus = inputbus
        })
    end
end

---返回recipe_name process_count complted_count
---@return string|nil, number?, number?
function ProcessControl.get_current_task()
    if #tasks > 0 then
        local task = tasks[1]
        local input_count = 0
        for _, v in pairs(task.will_input) do
            input_count = input_count + v
        end
        local inputbus_cout = 0
        for _, v in pairs(task.will_inputbus) do
            inputbus_cout = inputbus_cout + v
        end
        return task.recipe_name, input_count, inputbus_cout
    end
    return nil
end

function ProcessControl.get_task_count()
    return #tasks
end

function ProcessControl.realod_config()
    local config = Config.load()
    recipes = config:get_recipes()

    local transposer_uuids = config:get_transposer()
    for i, uuid in ipairs(transposer_uuids) do
        table.insert(transposers, component.proxy(uuid))
    end

    local chamber = config:get_chambers()
    for i, v in ipairs(chamber) do
        for j, v in ipairs(v) do
            chamber_inputs[v] = i
        end
    end
end

local function find_match_item_in_box(transposer, item_name)
    local can_find = false
    local match_item_slot = 1
    for item in transposer.getAllStacks(1) do
        if item.label == item_name then
            can_find = true
            break
        end
        match_item_slot = match_item_slot + 1
    end
    if not can_find then
        match_item_slot = nil
    end
    return can_find, match_item_slot
end

local function find_match_item_in_container(transposer, item_name)
    local can_find = false
    local match_item_slot = nil
    for i, item in ipairs(transposer.getFluidInTank(0)) do
        if item.label == item_name then
            can_find = true
            match_item_slot = i - 1
            break
        end
    end
    return can_find, match_item_slot
end

---@param callbeck fun(chamber_id)
function ProcessControl.update(callbeck)
    while #tasks > 0 do
        local has_trans = false
        local current_task = tasks[1]

        -- Trans Fluid
        for input_item_name, count in pairs(current_task.will_input) do
            local chamber_id = chamber_inputs[input_item_name]
            callbeck(chamber_id)

            local transposer = transposers[chamber_id]
            local find_result, match_item_slot = find_match_item_in_container(transposer, input_item_name)
            if find_result then
                local result, trans_to_input_count = transposer.transferFluid(0, 1, count, match_item_slot)
                has_trans = result

                local remain_count = count - trans_to_input_count
                if remain_count == 0 then
                    current_task.will_input[input_item_name] = nil
                else
                    current_task.will_input[input_item_name] = count - trans_to_input_count
                end
            end
        end
        -- Trans Item
        for inputbus_item_name, count in pairs(current_task.will_inputbus) do
            local transposer = transposers[7]
            local find_result, match_item_slot = find_match_item_in_box(transposer, inputbus_item_name)
            if find_result then
                local trans_to_inputbus_count = transposer.transferItem(1, 0, count, match_item_slot)
                if trans_to_inputbus_count > 0 then
                    has_trans = true
                end
                local remain_count = count - trans_to_inputbus_count
                if remain_count == 0 then
                    current_task.will_inputbus[inputbus_item_name] = nil
                else
                    current_task.will_inputbus[inputbus_item_name] = remain_count
                end
            end
        end

        if not next(current_task.will_input) and not next(current_task.will_inputbus) then
            table.remove(tasks, 1)
        end

        if not has_trans then
            break
        end
    end
    return true
end

return ProcessControl

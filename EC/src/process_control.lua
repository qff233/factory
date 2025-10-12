local Config = require("src.config")
local component = require("component")

---@class Task
---@field recipe_name string
---@field count_input table<string, number>
---@field remain_input table<string, number>
---@field count_inputbus table<string, number>
---@field remain_inputbus table<string, number>

---@class ProcessControl
---@field tasks Task[]
local ProcessControl = {}
local recipes = {}
local chamber_inputs = {}

---@type Task[]
local tasks = {}
---@type Transposer[]
local transposers = {}

local is_running = true
function ProcessControl.turnOn()
    is_running = true
end
function ProcessControl.turnOff()
    is_running = false
end
function ProcessControl.isRunning()
    return is_running
end

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

    local has_add_task = false
    local recipe = recipes[recipe_name]
    for _, task in ipairs(tasks) do
        if task.recipe_name == recipe_name then
            local has_add_task = true
            for item_name, item_count in pairs(recipe.inputs) do
                local remain_count = task.remain_input[item_name] or 0
                task.count_input[item_name] = task.count_input[item_name] + item_count * count
                task.remain_input[item_name] = remain_count + item_count * count
            end
            for item_name, item_count in pairs(recipe.inputbus) do
                local remain_count = task.remain_inputbus[item_name] or 0
                task.count_inputbus[item_name] = task.count_inputbus[item_name] + item_count * count
                task.remain_inputbus[item_name] = remain_count + item_count * count
            end
        end
    end

    if not has_add_task then
        ---@type Task
        local task = {
            recipe_name = recipe_name,
            count_input = {},
            remain_input = {},
            count_inputbus = {},
            remain_inputbus = {}
        }
        for item_name, item_count in pairs(recipe.inputs) do
            task.count_input[item_name] = item_count * count
            task.remain_input[item_name] = item_count * count
        end
        for item_name, item_count in pairs(recipe.inputbus) do
            task.count_inputbus[item_name] = item_count * count
            task.remain_inputbus[item_name] = item_count * count
        end
    end
end

---返回recipe_name process_count complted_count
---@return string|nil, number?, number?
function ProcessControl.get_current_task()
    if #tasks > 0 then
        local task = tasks[1]
        return task.recipe_name, task.remain_input, task.remain_inputbus
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
    if not is_running then
        return
    end

    --- TODO 添加机台状态  正在加工(记录当前程式，所有材料添加完后，禁止下一次加工，检测机台停止后切空闲)/空闲(开始下一次加工)

    while #tasks > 0 do
        ---@type Task
        local current_task = tasks[1]
        local chamber_ids = {}
        local has_trans = false

        -- Trans Fluid
        for item_name, count in pairs(current_task.remain_input) do
            local chamber_id = chamber_inputs[item_name]
            table.insert(chamber_ids, chamber_id)

            local transposer = transposers[chamber_id]
            local find_result, match_item_slot = find_match_item_in_container(transposer, item_name)
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
        callbeck(chamber_ids)

        -- Trans Item
        for item_name, count in pairs(current_task.remain_inputbus) do
            local transposer = transposers[7]
            local find_result, match_item_slot = find_match_item_in_box(transposer, item_name)
            if find_result then
                local trans_to_inputbus_count = transposer.transferItem(1, 0, count, match_item_slot)
                if trans_to_inputbus_count > 0 then
                    has_trans = true
                end
                local remain_count = count - trans_to_inputbus_count
                if remain_count == 0 then
                    current_task.will_inputbus[item_name] = nil
                else
                    current_task.will_inputbus[item_name] = remain_count
                end
            end
        end

        --- TODO 添加当前机台在线状态 OnlineRemote(远程下单) OnlineLocal(本地下单) Offline(不发送数据)
        --- TODO 禁止OnlineRemote时手动本地下单
        --- TODO 上传服务器当前Task状态
        if not next(current_task.will_input) and not next(current_task.will_inputbus) then
            table.remove(tasks, 1)
            --- TODO 开启设备禁止工作，读取工作状态 得到真正加工完成的时间
        end

        if not has_trans then
            break
        end
    end
end

return ProcessControl

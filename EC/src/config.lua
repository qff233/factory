local json = require("json")

---@class Config
---@field data table
local Config = {}
Config.__index = Config

function Config.load()
    ---@type Config
    local self = setmetatable({}, Config)
    local file = io.open("./config.json", "r")
    if file then
        local content = file:read("*a")
        local result, msg = json.decode(content)
        if not result then
            error(msg)
        end

        self.data = result
        file:close()
    end
    return self
end

---@return boolean, string
function Config:save()
    local result, msg = self:check()
    if not result then
        return result, msg
    end

    local file = io.open("./config.json", "w")
    file:write(json.encode(self.data))
    file:close()
    return true
end

function Config:check()
    local chambers = self.data.chambers
    local inputs = {}
    for i = 1, 6, 1 do
        for j, chamber_item in ipairs(chambers[i]) do
            if inputs[chamber_item] then
                return false, "流体配置必须唯一，`" .. chambers[i][j] .. "`已经配置过了！"
            end
            inputs[chamber_item] = true
        end
    end

    local recipes = self.data.recipes
    for name, recipe in pairs(recipes) do
        if recipe.inputs then
            for item_name, count in pairs(recipe.inputs) do
                if not inputs[item_name] then
                    return false, "`" .. name .. "`配方的输入仓必须包含配置过的流体！`" .. item_name ..
                        "` 没有包含在内！"
                end
            end
        end
    end
    return true
end

function Config:check_in_input_item(item_name)
    local chambers = self.data.chambers
    for i = 1, 6 do
        for j, lique_name in ipairs(chambers[i]) do
            if lique_name == item_name then
                return true, "[" .. i .. "]号已经有相同的流体`" .. chambers[i][j] .. "`!!"
            end
        end
    end
    return false
end

---@class Recipe
---@field inputs table<string, number>
---@field outputs table<string, number>
---@field inputbus table<string, number>
---@field outputbus table<string, number>

---@param reicipe_name string
---@return Recipe[]
function Config:get_recipes()
    return self.data.recipes
end

function Config:get_transposer()
    return self.data.transposer
end

function Config:get_recipe_input_counts(reicipe_name)
    local item_counts = {}
    local items = self.data.recipes[reicipe_name].inputs
    if items then
        for item_name, item_count in pairs(items) do
            table.insert(item_counts, item_name .. " " .. tostring(item_count))
        end
    end
    return item_counts
end

function Config:get_recipe_output_counts(reicipe_name)
    local item_counts = {}
    local items = self.data.recipes[reicipe_name].outputs
    if items then
        for item_name, item_count in pairs(items) do
            table.insert(item_counts, item_name .. " " .. tostring(item_count))
        end
    end
    return item_counts
end

function Config:get_recipe_inputbus_counts(reicipe_name)
    local item_counts = {}
    local items = self.data.recipes[reicipe_name].inputbus
    if items then
        for item_name, item_count in pairs(items) do
            table.insert(item_counts, item_name .. " " .. tostring(item_count))
        end
    end
    return item_counts
end

function Config:get_recipe_outputbus_counts(reicipe_name)
    local item_counts = {}
    local items = self.data.recipes[reicipe_name].outputbus
    if items then
        for item_name, item_count in pairs(items) do
            table.insert(item_counts, item_name .. " " .. tostring(item_count))
        end
    end
    return item_counts
end

function Config:set_recipe_input_counts(reicipe_name, item_name, item_count)
    if not self.data.recipes[reicipe_name].inputs then
        self.data.recipe[reicipe_name]["inputs"] = {}
    end
    self.data.recipes[reicipe_name].inputs[item_name] = item_count
end

function Config:set_recipe_output_counts(reicipe_name, item_name, item_count)
    if not self.data.recipes[reicipe_name].outputs then
        self.data.recipe[reicipe_name]["outputs"] = {}
    end
    self.data.recipes[reicipe_name].outputs[item_name] = item_count
end

function Config:set_recipe_inputbus_counts(reicipe_name, item_name, item_count)
    if not self.data.recipes[reicipe_name].inputbus then
        self.data.recipe[reicipe_name]["inputbus"] = {}
    end
    self.data.recipes[reicipe_name].inputbus[item_name] = item_count
end

function Config:set_recipe_outputbus_counts(reicipe_name, item_name, item_count)
    if not self.data.recipes[reicipe_name].outputbus then
        self.data.recipe[reicipe_name]["outputbus"] = {}
    end
    self.data.recipes[reicipe_name].outputbus[item_name] = item_count
end

---@return string[]
function Config:get_recipe_names()
    local recipe_names = {}
    for k, _ in pairs(self.data.recipes) do
        table.insert(recipe_names, k)
    end
    return recipe_names
end

function Config:add_recipes(recipe_name, recipe)
    self.data.recipes[recipe_name] = {
        inputs = {},
        outputs = {},
        inputbus = {},
        outputbus = {}
    }
end

function Config:del_recipes(recipe_name)
    self.data.recipes[recipe_name] = nil
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

---@return string[]
function Config:get_chamber(id)
    return deepcopy(self.data.chambers[id]) or {}
end

function Config:add_chamber_item(id, item_name)
    table.insert(self.data.chambers[id], item_name)
end

function Config:del_chamber_item(id, item_name)
    for i, v in ipairs(self.data.chambers[id]) do
        if v == item_name then
            table.remove(self.data.chambers[id], i)
        end
    end
end

---@return string[][]
function Config:get_chambers()
    return self.data.chambers
end

---@param id string
---@param chamber string[] 
---@return boolean, string
function Config:set_chamber(id, chamber)
    self.data.chambers[id] = chamber
    return self:check()
end

return Config

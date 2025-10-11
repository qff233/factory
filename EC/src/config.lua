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
        for j = 1, 7, 1 do
            local lique_name = chambers[i][j]
            if lique_name == "" then
                goto continue
            end
            if inputs[lique_name] then
                return false, "chambers config must unique. " .. chambers[i][j] .. " has same lique"
            end
            inputs[lique_name] = true

            ::continue::
        end
    end

    local recipes = self.data.recipes
    for name, recipe in pairs(recipes) do
        if recipe.inputs then
            for item_name, count in pairs(recipe.inputs) do
                if not inputs[item_name] then
                    return false,
                        "Inputs of `" .. name .. "` must contain chamber item! [" .. item_name .. "] can't contain"
                end
            end
        end
    end
    return true
end

function Config:check_in_input_item(item_name)
    local chambers = self.data.chambers
    local inputs = {}
    for i = 1, 6, 1 do
        for j = 1, 7, 1 do
            local lique_name = chambers[i][j]
            if lique_name == "" then
                goto continue
            end
            if inputs[lique_name] then
                return false, "chambers config must unique. " .. chambers[i][j] .. " has same lique"
            end
            inputs[lique_name] = true

            ::continue::
        end
    end
    if inputs[item_name] then
        return true
    else
        return false
    end
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

---@alias Chamber string[]

---@return Chamber
function Config:get_chamber(id)
    return self.data.chambers[id]
end

---@return Chamber[]
function Config:get_chambers()
    return self.data.chambers
end

---@param id string
---@param chamber Chamber
---@return boolean, string
function Config:set_chamber(id, chamber)
    self.data.chambers[id] = chamber
    return self:check()
end

return Config

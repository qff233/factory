local M = {}


---@class PriorityQueue
---@field heap {item: string, priority: number}[]
local PriorityQueue = {}
PriorityQueue.__index = PriorityQueue

---comment
---@return PriorityQueue
function M.new_priority_queue()
    local self = setmetatable({}, PriorityQueue)
    self.heap = {}
    return self
end

---comment
---@param item string
---@param priority number
function PriorityQueue:push(item, priority)
    table.insert(self.heap, { item = item, priority = priority })
    local i = #self.heap

    while i > 1 do
        local parent = math.floor(i / 2)
        if self.heap[parent].priority <= self.heap[i].priority then
            break
        end
        self.heap[parent], self.heap[i] = self.heap[i], self.heap[parent]
        i = parent
    end
end

function PriorityQueue:pop()
    if #self.heap == 0 then
        return nil
    end

    local min_item = self.heap[1].item
    self.heap[1] = self.heap[#self.heap]
    self.heap[#self.heap] = nil
    local n = #self.heap

    local i = 1
    while true do
        local left = 2 * i
        local right = 2 * i + 1
        local smallest = i

        if left <= n and self.heap[left].priority < self.heap[smallest].priority then
            smallest = left
        end

        if right <= n and self.heap[right].priority < self.heap[smallest].priority then
            smallest = right
        end

        if smallest == i then
            break
        end

        self.heap[i], self.heap[smallest] = self.heap[smallest], self.heap[i]
        i = smallest
    end

    return min_item
end

---comment
---@return boolean
function PriorityQueue:is_empty()
    return #self.heap == 0
end

---comment
---@param item string
---@return boolean
function PriorityQueue:contains(item)
    for _, v in ipairs(self.heap) do
        if v.item == item then
            return true
        end
    end
    return false
end

return M

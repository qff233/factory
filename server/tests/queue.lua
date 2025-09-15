local queue = require("queue")

local q = queue.new_priority_queue()

q:push("A", 9)
q:push("B", 8)
q:push("C", 7)
q:push("D", 6)
q:push("E", 5)

for _ = 1, 5 do
  print(q:pop())
end

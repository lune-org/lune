assert(_G ~= nil, "Missing _G")

assert(type(_G) == "table", "Invalid type for _G")

assert(_G.require == nil, "Built-in global value was found in _G")
assert(_G.print == nil, "Built-in global value was found in _G")
assert(_G.warn == nil, "Built-in global value was found in _G")
assert(_G.error == nil, "Built-in global value was found in _G")
assert(_G.coroutine == nil, "Built-in global value was found in _G")
assert(_G.typeof == nil, "Built-in global value was found in _G")
assert(_G.type == nil, "Built-in global value was found in _G")

assert(next(_G) == nil, "_G contained value but should be empty")

_G.Hello = "World"
assert(_G.Hello == "World", "Failed to set value in _G")

local bar = {}
_G.Foo = bar
assert(_G.Foo == bar, "Failed to set reference in _G")

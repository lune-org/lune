local module1
local module2

task.defer(function()
	module2 = require("./modules/async")
end)

task.spawn(function()
	module1 = require("./modules/async")
end)

task.wait(1)

assert(type(module1) == "table", "Required module1 did not return a table")
assert(module1.Foo == "Bar", "Required module1 did not contain correct values")
assert(module1.Hello == "World", "Required module1 did not contain correct values")

assert(type(module2) == "table", "Required module2 did not return a table")
assert(module2.Foo == "Bar", "Required module2 did not contain correct values")
assert(module2.Hello == "World", "Required module2 did not contain correct values")

assert(module1 == module2, "Required modules should point to the same return value")

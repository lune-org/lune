assert(#process.args > 0, "No process arguments found")

assert(process.args[1] == "Foo", "Invalid first argument to process")
assert(process.args[2] == "Bar", "Invalid second argument to process")

local success = pcall(function()
	process.args[1] = "abc"
end)
assert(not success, "Trying to set process arguments should throw an error")

local foundValue = false
for _, _ in process.args do
	foundValue = true
	break
end

assert(foundValue, "Iterating using generalized iteration")

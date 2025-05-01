local process = require("@lune/process")

assert(#process.args > 0, "No process arguments found")

assert(process.args[1] == "Foo", "Invalid first argument to process")
assert(process.args[2] == "Bar", "Invalid second argument to process")

local success, err = pcall(function()
	process.args[1] = "abc"
end)
local message = if err == nil then nil else tostring(err)
assert(
	success == false and type(message) == "string" and #message > 0,
	"Trying to set process arguments should throw an error with a message"
)
assert(
	string.find(message, "read") ~= nil,
	"Setting process args error message should mention that they are read-only"
)
assert(
	string.find(message, "only") ~= nil,
	"Setting process args error message should mention that they are read-only"
)

local foundValue = false
for _, _ in process.args do
	foundValue = true
	break
end

assert(foundValue, "Iterating using generalized iteration")

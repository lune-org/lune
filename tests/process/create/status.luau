local process = require("@lune/process")

local testCode = math.random(0, 255)
local testOk = testCode == 0

local exitChild = process.create("exit", { tostring(testCode) }, { shell = true })
local exitStatus = exitChild:status()

assert(type(exitStatus) == "table", "Child status should be a table")
assert(type(exitStatus.ok) == "boolean", "Child status.ok should be a boolean")
assert(type(exitStatus.code) == "number", "Child status.code should be a number")

assert(
	exitStatus.ok == testOk,
	"Child status should be "
		.. (if exitStatus.ok then "ok" else "not ok")
		.. ", was "
		.. (if exitStatus.ok then "not ok" else "ok")
)
assert(
	exitStatus.code == testCode,
	"Child process exited with an unexpected exit code!"
		.. `\nExpected: ${testCode}`
		.. `\nReceived: ${exitStatus.code}`
)

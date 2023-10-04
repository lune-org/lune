-- NOTE: luau-lsp warns without this for the roblox types below
--!nolint UnknownType

local task = require("@lune/task")

local function f() end

local thread1 = coroutine.create(f)
local thread2 = task.spawn(f)
local thread3 = task.defer(f)
local thread4 = task.delay(0, f)

assert(
	typeof(thread1) == "thread",
	"Calling typeof() did not return 'thread' after coroutine.create"
)
assert(typeof(thread2) == "thread", "Calling typeof() did not return 'thread' after task.spawn")
assert(typeof(thread3) == "thread", "Calling typeof() did not return 'thread' after task.defer")
assert(typeof(thread4) == "thread", "Calling typeof() did not return 'thread' after delay")

local roblox = require("@lune/roblox")

local rbx1 = roblox.Instance.new("Part")
local rbx2 = (roblox :: any).Vector3.zero
local rbx3 = (roblox :: any).Enum.KeyCode.A

assert(typeof(rbx1) == "Instance", "Calling typeof() did not return 'Instance' for Instance.new")
assert(typeof(rbx2) == "Vector3", "Calling typeof() did not return 'Vector3' for Vector3.zero")
assert(typeof(rbx3) == "EnumItem", "Calling typeof() did not return 'EnumItem' for Enum.KeyCode.A")

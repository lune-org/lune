local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

-- Issue #242: an instance reached via two different paths must be the *same*
-- userdata, so that it behaves correctly as a table key and with rawequal.

-- Via .Parent
local obj = Instance.new("Folder")
local child = Instance.new("Folder")
child.Parent = obj

assert(child.Parent == obj, "Parent should be equal to the parent instance")
assert(rawequal(child.Parent, obj), "Parent should be the *same* userdata as the parent instance")

local byKey = { [obj] = 100 }
assert(byKey[obj] == 100)
assert(byKey[child.Parent] == 100, "instance reached via .Parent must work as the same table key")

-- Via GetChildren / FindFirstChild
assert(rawequal(obj:GetChildren()[1], child), "GetChildren must return the canonical userdata")
assert(
	rawequal(obj:FindFirstChild("Folder"), child),
	"FindFirstChild must return the canonical userdata"
)

-- Via ref-typed properties (Motor6D.Part0 example from the issue thread)
local part = Instance.new("Part")
local motorA = Instance.new("Motor6D")
local motorB = Instance.new("Motor6D")
motorA.Part0 = part
motorB.Part0 = part

assert(motorA.Part0 == part)
assert(rawequal(motorA.Part0, part), "a ref'd instance must be the canonical userdata")
assert(
	rawequal(motorA.Part0, motorB.Part0),
	"the same ref'd instance reached from two places must be the same userdata"
)

local dict = {}
dict[motorA.Part0] = 1
dict[motorB.Part0] = 2
local count = 0
for _ in dict do
	count += 1
end
assert(count == 1, "two references to the same instance must collapse to one table key")
assert(dict[part] == 2, "the collapsed table key must resolve to the latest assigned value")

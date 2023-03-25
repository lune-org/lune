local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local part = Instance.new("Part")
local workspace = Instance.new("Workspace")

-- Valid

assert(part:IsA("Part") == true)
assert(part:IsA("BasePart") == true)
assert(part:IsA("PVInstance") == true)
assert(part:IsA("Instance") == true)

assert(workspace:IsA("Workspace") == true)
assert(workspace:IsA("Model") == true)
assert(workspace:IsA("Instance") == true)

-- Invalid

assert(part:IsA("part") == false)
assert(part:IsA("Base-Part") == false)
assert(part:IsA("Model") == false)
assert(part:IsA("Paart") == false)

assert(workspace:IsA("Service") == false)
assert(workspace:IsA(".") == false)
assert(workspace:IsA("") == false)

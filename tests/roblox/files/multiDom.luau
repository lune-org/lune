local fs = require("@lune/fs")
local roblox = require("@lune/roblox") :: any

-- Issue #356: deserializing the same place twice must produce two fully
-- independent doms. Their instances share referents (and UniqueIds), so an
-- implementation that merged everything into one dom would confuse them.

local bytes = fs.readFile("tests/roblox/rbx-test-files/places/baseplate-566/binary.rbxl")

local domA = roblox.deserializePlace(bytes)
local domB = roblox.deserializePlace(bytes)

assert(domA ~= domB, "two separately deserialized places must be distinct instances")
assert(not rawequal(domA, domB))

local wsA = assert(domA:FindFirstChild("Workspace"))
local wsB = assert(domB:FindFirstChild("Workspace"))

assert(wsA ~= wsB, "instances from different doms must not be equal even with colliding referents")
assert(not rawequal(wsA, wsB))

-- UniqueId must read correctly and independently in each dom
local expected = "44b188dace632b4702e9c68d004815fc"
assert(tostring(wsA.UniqueId) == expected, "UniqueId must be read correctly in dom A")
assert(tostring(wsB.UniqueId) == expected, "UniqueId must be read correctly in dom B")

-- Within a single dom, identity must still hold across lookups
assert(rawequal(domA:FindFirstChild("Workspace"), wsA))

-- Editing one dom must not affect the other
wsA.Name = "RenamedWorkspace"
assert(wsA.Name == "RenamedWorkspace")
assert(wsB.Name == "Workspace", "editing one dom must not leak into another")

-- Destroying the root of a dom tears the whole dom down
domA:Destroy()
assert(not pcall(function()
	return wsA.Name
end), "instances of a destroyed dom must error when accessed")
-- The other dom is unaffected
assert(wsB.Name == "Workspace", "destroying one dom must not affect another")

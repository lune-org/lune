local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Model")
local child = Instance.new("Part")
local objValue1 = Instance.new("ObjectValue")
local objValue2 = Instance.new("ObjectValue")

objValue1.Name = "ObjectValue1"
objValue2.Name = "ObjectValue2"
(objValue1 :: any).Value = root;
(objValue2 :: any).Value = child
objValue1.Parent = child
objValue2.Parent = child
child.Parent = root

local clonedChild = child:Clone()
assert(clonedChild ~= child)
assert(clonedChild.Parent == nil)

local clonedObjValue1 = clonedChild[objValue1.Name]
local clonedObjValue2 = clonedChild[objValue2.Name]

assert(clonedObjValue1 ~= objValue1)
assert(clonedObjValue2 ~= objValue2)

assert(clonedObjValue1.Value == root, "ObjectValue1.Value should still point to original root")
assert(clonedObjValue2.Value == clonedChild, "ObjectValue2.Value should point to cloned child")

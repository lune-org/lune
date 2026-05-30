--[[
	Shared fixtures for the Instance:QueryDescendants tests.

	The tree and the expected results below are taken directly from probes run
	against the live Roblox engine, so these tests assert real engine behavior:

		QDRoot (Folder)
		|- ModelA (Model)        [tag Apple, attr Variety=Fuji, Flavor=4]
		|  |- RedTree (Part)     [CanCollide=false, tag Fruit]
		|  '- MeshA (MeshPart)   [tag SwordPart]
		|- FolderB (Folder)
		|  '- ModelC (Model)
		|     '- PartC (Part)    [default CanCollide=true]
		'- PartD (Part)          [attr OnFire=true]
]]

local roblox = require("@lune/roblox")
local Instance = roblox.Instance
local Vector3 = (roblox :: any).Vector3

local function build(): any
	local root = Instance.new("Folder")
	root.Name = "QDRoot"

	local modelA = Instance.new("Model") :: any
	modelA.Name = "ModelA"
	modelA.Parent = root
	modelA:AddTag("Apple")
	modelA:SetAttribute("Variety", "Fuji")
	modelA:SetAttribute("Flavor", 4)
	modelA:SetAttribute("Vec", Vector3.new(1, 2, 3))

	local redTree = Instance.new("Part") :: any
	redTree.Name = "RedTree"
	redTree.CanCollide = false
	redTree.Parent = modelA
	redTree:AddTag("Fruit")

	local meshA = Instance.new("MeshPart") :: any
	meshA.Name = "MeshA"
	meshA.Parent = modelA
	meshA:AddTag("SwordPart")

	local folderB = Instance.new("Folder") :: any
	folderB.Name = "FolderB"
	folderB.Parent = root

	local modelC = Instance.new("Model") :: any
	modelC.Name = "ModelC"
	modelC.Parent = folderB

	local partC = Instance.new("Part") :: any
	partC.Name = "PartC"
	partC.Parent = modelC

	local partD = Instance.new("Part") :: any
	partD.Name = "PartD"
	partD.Parent = root
	partD:SetAttribute("OnFire", true)

	return root
end

local function namesOf(instances): { string }
	local names = {}
	for _, inst in instances do
		table.insert(names, inst.Name)
	end
	return names
end

local function arraysEqual(a: { string }, b: { string }): boolean
	if #a ~= #b then
		return false
	end
	for i = 1, #a do
		if a[i] ~= b[i] then
			return false
		end
	end
	return true
end

-- Asserts the selector returns exactly the expected names, in the expected order
local function check(root: any, selector: string, expected: { string })
	local got = namesOf(root:QueryDescendants(selector))
	assert(
		arraysEqual(got, expected),
		`selector '{selector}': expected [{table.concat(expected, ", ")}], got [{table.concat(
			got,
			", "
		)}]`
	)
end

-- Asserts the selector raises an error
local function checkError(root: any, selector: string)
	local ok = pcall(function()
		return root:QueryDescendants(selector)
	end)
	assert(not ok, `selector '{selector}': expected an error, but it succeeded`)
end

return {
	build = build,
	check = check,
	checkError = checkError,
}

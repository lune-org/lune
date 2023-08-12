local luau = require("@lune/luau")

local EMPTY_LUAU_CODE_BLOCK = "do end"

local MIN_OPTION_VALUE = 0
local MAX_OPTION_VALUE = 2
local OPTION_NAMES = {
	"optimizationLevel",
	"coverageLevel",
	"debugLevel",
}

for _, optionName in OPTION_NAMES do
	-- In range should work
	for optionValue = MIN_OPTION_VALUE, MAX_OPTION_VALUE, 1 do
		local options = { [optionName] = optionValue }
		local success2 = pcall(function()
			luau.compile(EMPTY_LUAU_CODE_BLOCK, options)
		end)
		assert(success2, "expected `luau.compile` to accept options within range")
	end
	-- Lower than min range should error
	for optionValue = MIN_OPTION_VALUE - 16, MIN_OPTION_VALUE - 1, 1 do
		local options = { [optionName] = optionValue }
		local success2 = pcall(function()
			luau.compile(EMPTY_LUAU_CODE_BLOCK, options)
		end)
		assert(not success2, "expected `luau.compile` to not accept options outside of range")
	end
	-- Higher than max range should error
	for optionValue = MAX_OPTION_VALUE + 1, MAX_OPTION_VALUE + 16, 1 do
		local options = { [optionName] = optionValue }
		local success2 = pcall(function()
			luau.compile(EMPTY_LUAU_CODE_BLOCK, options)
		end)
		assert(not success2, "expected `luau.compile` to not accept options outside of range")
	end
end

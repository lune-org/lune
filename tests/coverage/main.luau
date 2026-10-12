local luau = require("@lune/luau")

local module = require("./module")

for _ = 1, 3 do
	assert(module.double(2) == 4, "expected module.double to double")
end

local loaded = luau.load(
	"local value = 1\nif value > 1 then\n\tvalue = 2\nend\nreturn value",
	{ debugName = "loaded" }
)
assert(loaded() == 1, "expected the loaded chunk to return 1")

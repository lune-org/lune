local serde = require("@lune/serde")

-- Generate testing data

local binaryBlob = ""
for _ = 1, 1024 do
	binaryBlob ..= string.char(math.random(1, 127))
end

local jsonBlob = serde.encode("json", {
	Foo = "Bar",
	Hello = "World",
	Inner = { Array = { 1, 2, 3 } },
}, true)

-- Return testing data and utils

return {
	binaryBlob = binaryBlob,
	jsonBlob = jsonBlob,
}

local serde = require("@lune/serde")

local value = serde.decode(
	"yaml",
	[[
single: '1'
double: "1"
tagged: !!str 1
booleanString: 'true'
nullString: 'null'
integer: 1
float: 1.5
boolean: true
nullValue: null
]]
)

assert(typeof(value.single) == "string" and value.single == "1")
assert(typeof(value.double) == "string" and value.double == "1")
assert(typeof(value.tagged) == "string" and value.tagged == "1")
assert(typeof(value.booleanString) == "string" and value.booleanString == "true")
assert(typeof(value.nullString) == "string" and value.nullString == "null")
assert(typeof(value.integer) == "number" and value.integer == 1)
assert(typeof(value.float) == "number" and value.float == 1.5)
assert(typeof(value.boolean) == "boolean" and value.boolean == true)
assert(value.nullValue == nil)

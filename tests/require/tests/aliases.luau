local module = require("@tests/require/tests/module")

assert(type(module) == "table", "Required module did not return a table")
assert(module.Foo == "Bar", "Required module did not contain correct values")
assert(module.Hello == "World", "Required module did not contain correct values")

local module2 = require("@require-tests/module")

assert(type(module2) == "table", "Required module did not return a table")
assert(module2.Foo == "Bar", "Required module did not contain correct values")
assert(module2.Hello == "World", "Required module did not contain correct values")

assert(module == module2, "Require did not return the same table for the same module")

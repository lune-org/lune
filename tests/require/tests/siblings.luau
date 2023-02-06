local module = require("./module")

assert(type(module) == "table", "Required module did not return a table")
assert(module.Foo == "Bar", "Required module did not contain correct values")
assert(module.Hello == "World", "Required module did not contain correct values")

require("./children")
require("./parents")

require("children")
require("parents")

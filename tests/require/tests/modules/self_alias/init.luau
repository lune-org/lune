local outer = require("./module")
local inner = require("@self/module")

assert(type(outer) == "table", "Outer module is not a table")
assert(type(inner) == "table", "Inner module is not a table")

assert(outer.Foo == inner.Foo, "Outer and inner modules have different Foo values")
assert(inner.Bar == outer.Bar, "Outer and inner modules have different Bar values")

return inner

local inner = require("@self/module") :: any -- FIXME: luau-lsp does not yet support self alias
local outer = require("./module")

assert(type(outer) == "table", "Outer module is not a table")
assert(type(inner) == "table", "Inner module is not a table")

assert(outer.Foo == inner.Foo, "Outer and inner modules have different Foo values")
assert(inner.Hello == outer.Hello, "Outer and inner modules have different Hello values")

return inner

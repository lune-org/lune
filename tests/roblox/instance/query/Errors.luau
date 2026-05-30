local query = require("./tree")
local root = query.build()

-- Empty / whitespace-only selectors are not errors; they match nothing
query.check(root, "", {})
query.check(root, "   ", {})

-- Comparing a property/attribute of an unsupported type errors
query.checkError(root, "[Color = 1]") -- Color3 property
query.checkError(root, "[$Vec = 1]") -- Vector3 attribute

-- Malformed bracket selectors
query.checkError(root, "[Prop]") -- property requires "= value"
query.checkError(root, "[Name = 'RedTree']") -- single quotes are not valid
query.checkError(root, "[Foo = 4.0]") -- "." is not a value character
query.checkError(root, "[Transparency = 0.5]")
query.checkError(root, "[CanCollide = false") -- unterminated bracket

-- Unsupported / malformed pseudo-classes and combinators
query.checkError(root, ":Hover")
query.checkError(root, ":foo(Part)")
query.checkError(root, ":not(SpotLight") -- unterminated paren
query.checkError(root, "*")
query.checkError(root, "Part + Part")
query.checkError(root, "Model >") -- combinator with no following compound

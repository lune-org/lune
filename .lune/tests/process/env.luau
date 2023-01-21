local randomKey = string.format("LUNE_TEST_%d", math.random(1, 999_999))

assert(process.env[randomKey] == nil, "Unset variable returned a non-nil value")

process.env[randomKey] = "abc"
assert(process.env[randomKey] == "abc", "Failed to set environment variable")

process.env[randomKey] = nil
assert(process.env[randomKey] == nil, "Failed to set environment variable")

local foundValue = false
for _, _ in process.env do
	foundValue = true
	break
end

assert(foundValue, "Iterating using generalized iteration")

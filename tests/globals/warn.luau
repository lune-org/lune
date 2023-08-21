assert(warn ~= nil, "Missing 'warn' global")
assert(
	type(warn) == "function",
	string.format("Global 'warn' should be a function, got '%s'", tostring(type(warn)))
)

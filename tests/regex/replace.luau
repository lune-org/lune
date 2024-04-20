local regex = require("@lune/regex")

-- Tests taken from the Regex crate

local function replace(
	name: string,
	pattern: string,
	find: string,
	replace: string,
	expected: string
)
	local re = regex.new(pattern)
	local replaced = re:replace(find, replace)
	if replaced ~= expected then
		error(`test '{name}' did not return expected result (expected {expected} got {replaced})`)
	end
end

local function replaceAll(
	name: string,
	pattern: string,
	find: string,
	replace: string,
	expected: string
)
	local re = regex.new(pattern)
	local replaced = re:replaceAll(find, replace)
	if replaced ~= expected then
		error(`test '{name}' did not return expected result (expected {expected} got {replaced})`)
	end
end

replace("first", "[0-9]", "age: 26", "Z", "age: Z6")
replace("plus", "[0-9]+", "age: 26", "Z", "age: Z")
replaceAll("all", "[0-9]", "age: 26", "Z", "age: ZZ")
replace("groups", "([^ ]+)[ ]+([^ ]+)", "w1 w2", "$2 $1", "w2 w1")
replace("double dollar", "([^ ]+)[ ]+([^ ]+)", "w1 w2", "$2 $$1", "w2 $1")

replaceAll(
	"named",
	"(?P<first>[^ ]+)[ ]+(?P<last>[^ ]+)(?P<space>[ ]*)",
	"w1 w2 w3 w4",
	"$last $first$space",
	"w2 w1 w4 w3"
)
replaceAll("trim", "^[ \t]+|[ \t]+$", " \t  trim me\t   \t", "", "trim me")
replace("number hypen", "(.)(.)", "ab", "$1-$2", "a-b")
replaceAll("simple expand", "([a-z]) ([a-z])", "a b", "$2 $1", "b a")
replaceAll("literal dollar 1", "([a-z]+) ([a-z]+)", "a b", "$$1", "$1")
replaceAll("literal dollar 2", "([a-z]+) ([a-z]+)", "a b", "$2 $$c $1", "b $c a")

replaceAll("match at start replace with empty", "foo", "foobar", "", "bar")
replace("single empty match", "^", "bar", "foo", "foobar")

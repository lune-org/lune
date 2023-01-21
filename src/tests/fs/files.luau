-- Generate test data & make sure our bin dir exists

local binary = ""
for _ = 1, 1024 do
	binary ..= string.char(math.random(1, 127))
end

local json = net.jsonEncode({
	Foo = "Bar",
	Hello = "World",
	Inner = { Array = { 1, 2, 3 } },
}, true)

fs.writeDir("bin")

-- Write both of our files

fs.writeFile("bin/test_binary", binary)
fs.writeFile("bin/test_json.json", json)

-- Make sure reading the file we just
-- wrote gets us back the original strings

assert(
	fs.readFile("bin/test_binary") == binary,
	"Binary file round-trip resulted in different strings"
)

assert(
	fs.readFile("bin/test_json.json") == json,
	"JSON file round-trip resulted in different strings"
)

-- Make sure file checks succeed but dir checks fail

assert(fs.isFile("bin/test_binary"), "Binary file isFile check failed")
assert(fs.isFile("bin/test_json.json"), "JSON file isFile check failed")

assert(not fs.isDir("bin/test_binary"), "Binary file isDir check failed")
assert(not fs.isDir("bin/test_json.json"), "JSON file isDir check failed")

-- Remove the files and make sure
-- the APIs say they no longer exist

fs.removeFile("bin/test_binary")
fs.removeFile("bin/test_json.json")

assert(not fs.isDir("bin/test_binary"), "Binary after removal isDir check failed")
assert(not fs.isFile("bin/test_binary"), "Binary after removal isFile check failed")

assert(not fs.isDir("bin/test_json.json"), "JSON after removal isDir check failed")
assert(not fs.isFile("bin/test_json.json"), "JSON after removal isFile check failed")

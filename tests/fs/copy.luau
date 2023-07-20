local TEMP_DIR_PATH = "bin/"
local TEMP_ROOT_PATH = TEMP_DIR_PATH .. "fs_copy_test"
local TEMP_ROOT_PATH_2 = TEMP_DIR_PATH .. "fs_copy_test_2"

local fs = require("@lune/fs")

-- Generate test data & make sure our bin dir exists

local binary = ""
for _ = 1, 1024 do
	binary ..= string.char(math.random(1, 127))
end

fs.writeDir(TEMP_DIR_PATH)
if fs.isDir(TEMP_ROOT_PATH) then
	fs.removeDir(TEMP_ROOT_PATH)
end
if fs.isDir(TEMP_ROOT_PATH_2) then
	fs.removeDir(TEMP_ROOT_PATH_2)
end

--[[
	Create a file structure like this:

	-> fs_copy_test
	-- -> foo (dir)
	-- -- -> bar (dir)
	-- -- -- -> baz (file)
	-- -- -> fizz (file)
	-- -- -> buzz (file)

]]

fs.writeDir(TEMP_ROOT_PATH)
fs.writeDir(TEMP_ROOT_PATH .. "/foo")
fs.writeDir(TEMP_ROOT_PATH .. "/foo/bar")
fs.writeFile(TEMP_ROOT_PATH .. "/foo/bar/baz", binary)
fs.writeFile(TEMP_ROOT_PATH .. "/foo/fizz", binary)
fs.writeFile(TEMP_ROOT_PATH .. "/foo/buzz", binary)

-- Copy the entire structure

fs.copy(TEMP_ROOT_PATH, TEMP_ROOT_PATH_2)

-- Verify the copied structure

assert(fs.isDir(TEMP_ROOT_PATH_2), "Missing copied dir - root/")
assert(fs.isDir(TEMP_ROOT_PATH_2 .. "/foo"), "Missing copied dir - root/foo/")
assert(fs.isDir(TEMP_ROOT_PATH_2 .. "/foo/bar"), "Missing copied dir - root/foo/bar/")
assert(fs.isFile(TEMP_ROOT_PATH_2 .. "/foo/bar/baz"), "Missing copied file - root/foo/bar/baz")
assert(fs.isFile(TEMP_ROOT_PATH_2 .. "/foo/fizz"), "Missing copied file - root/foo/fizz")
assert(fs.isFile(TEMP_ROOT_PATH_2 .. "/foo/buzz"), "Missing copied file - root/foo/buzz")

-- Make sure the copied files are correct

assert(
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/bar/baz") == binary,
	"Invalid copied file - root/foo/bar/baz"
)
assert(
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/fizz") == binary,
	"Invalid copied file - root/foo/fizz"
)
assert(
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/buzz") == binary,
	"Invalid copied file - root/foo/buzz"
)

-- Finally, clean up after us for any subsequent tests

fs.removeDir(TEMP_ROOT_PATH)
fs.removeDir(TEMP_ROOT_PATH_2)

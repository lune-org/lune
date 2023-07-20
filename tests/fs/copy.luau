local TEMP_DIR_PATH = "bin/"
local TEMP_ROOT_PATH = TEMP_DIR_PATH .. "fs_copy_test"
local TEMP_ROOT_PATH_2 = TEMP_DIR_PATH .. "fs_copy_test_2"

local fs = require("@lune/fs")
local utils = require("./utils")

-- Make sure our bin dir exists

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
fs.writeFile(TEMP_ROOT_PATH .. "/foo/bar/baz", utils.binaryBlob)
fs.writeFile(TEMP_ROOT_PATH .. "/foo/fizz", utils.binaryBlob)
fs.writeFile(TEMP_ROOT_PATH .. "/foo/buzz", utils.binaryBlob)

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
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/bar/baz") == utils.binaryBlob,
	"Invalid copied file - root/foo/bar/baz"
)
assert(
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/fizz") == utils.binaryBlob,
	"Invalid copied file - root/foo/fizz"
)
assert(
	fs.readFile(TEMP_ROOT_PATH_2 .. "/foo/buzz") == utils.binaryBlob,
	"Invalid copied file - root/foo/buzz"
)

-- Finally, clean up after us for any subsequent tests

fs.removeDir(TEMP_ROOT_PATH)
fs.removeDir(TEMP_ROOT_PATH_2)

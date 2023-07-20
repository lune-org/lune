local TEMP_DIR_PATH = "bin/"
local TEMP_ROOT_PATH = TEMP_DIR_PATH .. "fs_dirs_test"

local fs = require("@lune/fs")

-- Write two inner dirs in the bin dir, a parent and a child

fs.writeDir(TEMP_ROOT_PATH .. "/test_inner")

-- Make sure dir checks succeed but file
-- checks fail for all levels of dirs

assert(fs.isDir(TEMP_DIR_PATH), "Dir root isDir check failed")
assert(fs.isDir(TEMP_ROOT_PATH), "Dir outer isDir check failed")
assert(fs.isDir(TEMP_ROOT_PATH .. "/test_inner"), "Dir inner isDir check failed")

assert(not fs.isFile(TEMP_DIR_PATH), "Dir root isFile check failed")
assert(not fs.isFile(TEMP_ROOT_PATH), "Dir outer isFile check failed")
assert(not fs.isFile(TEMP_ROOT_PATH .. "/test_inner"), "Dir inner isFile check failed")

-- Remove the created parent and child dirs and
-- make sure the APIs say they no longer exist

fs.removeDir(TEMP_ROOT_PATH)

assert(not fs.isDir(TEMP_ROOT_PATH), "After removal isDir check failed")
assert(not fs.isFile(TEMP_ROOT_PATH), "After removal isFile check failed")

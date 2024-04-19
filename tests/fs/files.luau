local TEMP_DIR_PATH = "bin/"
local TEMP_ROOT_PATH = TEMP_DIR_PATH .. "fs_files_test"

local fs = require("@lune/fs")
local utils = require("./utils")

-- Make sure our bin dir exists

fs.writeDir(TEMP_DIR_PATH)
fs.writeDir(TEMP_ROOT_PATH)

-- Write both of our files

fs.writeFile(TEMP_ROOT_PATH .. "/test_binary", utils.binaryBlob)
fs.writeFile(TEMP_ROOT_PATH .. "/test_json.json", utils.jsonBlob)

-- Make sure reading the file we just
-- wrote gets us back the original strings

assert(
	fs.readFile(TEMP_ROOT_PATH .. "/test_binary") == utils.binaryBlob,
	"Binary file round-trip resulted in different strings"
)

assert(
	fs.readFile(TEMP_ROOT_PATH .. "/test_json.json") == utils.jsonBlob,
	"JSON file round-trip resulted in different strings"
)

-- Make sure file checks succeed but dir checks fail

assert(fs.isFile(TEMP_ROOT_PATH .. "/test_binary"), "Binary file isFile check failed")
assert(fs.isFile(TEMP_ROOT_PATH .. "/test_json.json"), "JSON file isFile check failed")

assert(not fs.isDir(TEMP_ROOT_PATH .. "/test_binary"), "Binary file isDir check failed")
assert(not fs.isDir(TEMP_ROOT_PATH .. "/test_json.json"), "JSON file isDir check failed")

-- Remove the files and make sure
-- the APIs say they no longer exist

fs.removeFile(TEMP_ROOT_PATH .. "/test_binary")
fs.removeFile(TEMP_ROOT_PATH .. "/test_json.json")

assert(not fs.isDir(TEMP_ROOT_PATH .. "/test_binary"), "Binary after removal isDir check failed")
assert(not fs.isFile(TEMP_ROOT_PATH .. "/test_binary"), "Binary after removal isFile check failed")

assert(not fs.isDir(TEMP_ROOT_PATH .. "/test_json.json"), "JSON after removal isDir check failed")
assert(not fs.isFile(TEMP_ROOT_PATH .. "/test_json.json"), "JSON after removal isFile check failed")

-- Remove the testing dir specific to this test

fs.removeDir(TEMP_ROOT_PATH)

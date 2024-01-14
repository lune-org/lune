--!nolint UnknownType

local TEMP_DIR_PATH = "bin/"
local TEMP_FILE_PATH = TEMP_DIR_PATH .. "metadata_test"

local fs = require("@lune/fs")
local task = require("@lune/task")
local utils = require("./utils")

-- Make sure our bin dir exists

fs.writeDir(TEMP_DIR_PATH)
if fs.isFile(TEMP_FILE_PATH) then
	fs.removeFile(TEMP_FILE_PATH)
end

--[[
	1. File should initially not exist
	2. Write the file
	3. File should now exist
]]

assert(not fs.metadata(TEMP_FILE_PATH).exists, "File metadata not exists failed")
fs.writeFile(TEMP_FILE_PATH, utils.binaryBlob)
assert(fs.metadata(TEMP_FILE_PATH).exists, "File metadata exists failed")

--[[
	1. Kind should be `dir` for our temp directory
	2. Kind should be `file` for our temp file
]]

local metaDir = fs.metadata(TEMP_DIR_PATH)
local metaFile = fs.metadata(TEMP_FILE_PATH)
assert(metaDir.kind == "dir", "Dir metadata kind was invalid")
assert(metaFile.kind == "file", "File metadata kind was invalid")

--[[
	1. Capture initial metadata
	2. Wait for a bit so that timestamps can change
	3. Write the file, with an extra newline
	4. Metadata changed timestamp should be different
	5. Metadata created timestamp should be the same
	6. Timestamps should be DateTime structs
]]

local metaBefore = fs.metadata(TEMP_FILE_PATH)
task.wait(1)
fs.writeFile(TEMP_FILE_PATH, utils.binaryBlob .. "\n")
local metaAfter = fs.metadata(TEMP_FILE_PATH)

assert(
	metaAfter.modifiedAt ~= metaBefore.modifiedAt,
	"File metadata change timestamp did not change"
)
assert(
	metaAfter.createdAt == metaBefore.createdAt,
	"File metadata creation timestamp changed from modification"
)

assert(typeof(metaAfter.modifiedAt) == "DateTime", "File metadata modifiedAt is not a DateTime")
assert(typeof(metaAfter.createdAt) == "DateTime", "File metadata createdAt is not a DateTime")

--[[
	1. Permissions should exist
	2. Our newly created file should not be readonly
]]
assert(metaAfter.permissions ~= nil, "File metadata permissions are missing")
assert(not metaAfter.permissions.readOnly, "File metadata permissions are readonly")

-- Finally, clean up after us for any subsequent tests

fs.removeFile(TEMP_FILE_PATH)

--> A utility script that prints out a CSV
--> file in a prettified format to stdout

local LINE_SEPARATOR = "\n"
local COMMA_SEPARATOR = ","

local fs = require("@lune/fs")
local process = require("@lune/process")

local path = process.args[1] or ".lune/data/test.csv"

assert(path ~= nil and #path > 0, "No input file path was given")
assert(not fs.isDir(path), "Input file path was a dir, not a file")
assert(fs.isFile(path), "Input file path does not exist")

-- Read all the lines of the wanted file, and then split
-- out the raw lines containing comma-separated values

local csvTable = {}
for index, rawLine in string.split(fs.readFile(path), LINE_SEPARATOR) do
	if #rawLine > 0 then
		csvTable[index] = string.split(rawLine, COMMA_SEPARATOR)
	end
end

-- Gather the maximum widths of strings
-- for alignment & spacing in advance

local maxWidths = {}
for _, row in csvTable do
	for index, value in row do
		maxWidths[index] = math.max(maxWidths[index] or 0, #value)
	end
end

local totalWidth = 0
local totalColumns = 0
for _, width in maxWidths do
	totalWidth += width
	totalColumns += 1
end

-- We have everything we need, print it out with
-- the help of some unicode box drawing characters

local thiccLine = string.rep("━", totalWidth + totalColumns * 3 - 1)

print(string.format("┏%s┓", thiccLine))

for rowIndex, row in csvTable do
	local paddedValues = {}
	for valueIndex, value in row do
		local spacing = string.rep(" ", maxWidths[valueIndex] - #value)
		table.insert(paddedValues, value .. spacing)
	end
	print(string.format("┃ %s ┃", table.concat(paddedValues, " ┃ ")))
	-- The first line is the header, we should
	-- print out an extra separator below it
	if rowIndex == 1 then
		print(string.format("┣%s┫", thiccLine))
	end
end
print(string.format("┗%s┛", thiccLine))
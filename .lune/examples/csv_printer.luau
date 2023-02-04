--> A utility script that prints out a CSV
--> file in a prettified format to stdout

local LINE_SEPARATOR = "\n"
local COMMA_SEPARATOR = ","

local path = process.args[1] or ".lune/data/test.csv"

assert(path ~= nil and #path > 0, "No input file path was given")
assert(not fs.isDir(path), "Input file path was a dir, not a file")
assert(fs.isFile(path), "Input file path does not exist")

-- Read all the lines of the wanted file
local rawLines = string.split(fs.readFile(path), LINE_SEPARATOR)

-- Split the raw lines into header and table of data
local csvHeader = string.split(rawLines[1], COMMA_SEPARATOR)
local csvTable = {}
for index = 2, #rawLines, 1 do -- NOTE: We skip the first line here, that's the header
	csvTable[index - 1] = string.split(rawLines[index], COMMA_SEPARATOR)
end

-- Gather the maximum widths of strings for alignment
local maxWidths = {}

for index, header in csvHeader do
	maxWidths[index] = #header
end

for _, row in csvTable do
	for index, value in row do
		maxWidths[index] = math.max(maxWidths[index], #value)
	end
end

local totalWidth = 0
for _, width in maxWidths do
	totalWidth += width
end

-- Print it all out

local function printRow(row: { string })
	local paddedValues = {}
	for index, value in row do
		local spacing = string.rep(" ", maxWidths[index] - #value)
		table.insert(paddedValues, string.format(" %s%s ", value, spacing))
	end
	print(string.format("┃ %s ┃", table.concat(paddedValues, "┃")))
end

local thiccLine = string.rep("━", totalWidth + #csvHeader * 3 + 1)
print(string.format("┏%s┓", thiccLine))
printRow(csvHeader)
print(string.format("┣%s┫", thiccLine))
for _, row in csvTable do
	printRow(row)
end
print(string.format("┗%s┛", thiccLine))

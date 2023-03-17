-- NOTE: This must be ran in Roblox Studio to get up-to-date font values

local contents = ""

contents ..= "\ntype FontData = (&'static str, FontWeight, FontStyle);\n"

-- FontWeight enum and implementation

local function makeRustEnum(enum, numType: string)
	local name = string.gsub(tostring(enum), "^Enum.", "")

	-- TODO: Use FromStr and AsStr traits for name functions
	-- TODO: Match the naming convention that rbx-dom has for number/value functions

	local result = ""

	result ..= "\n#[derive(Debug, Clone, PartialEq)]"
	result ..= "\npub(crate) enum " .. name .. " {"
	for _, enum in enum:GetEnumItems() do
		result ..= string.format("\n    %s,", enum.Name)
	end
	result ..= "\n}\n"

	result ..= "\nimpl " .. name .. " {"
	result ..= "\n    pub fn into_name(self) -> &'static str {"
	result ..= "\n        match self {"
	for _, enum in enum:GetEnumItems() do
		result ..= string.format('\n            Self::%s => "%s",', enum.Name, enum.Name)
	end
	result ..= "\n        }"
	result ..= "\n    }"
	result ..= "\n"
	result ..= "\n    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {"
	result ..= "\n        match name.as_ref() {"
	for _, enum in enum:GetEnumItems() do
		result ..= string.format('\n            "%s" => Some(Self::%s),', enum.Name, enum.Name)
	end
	result ..= "\n            _ => None,"
	result ..= "\n        }"
	result ..= "\n    }"
	result ..= "\n"
	result ..= "\n    pub fn into_num(self) -> " .. numType .. " {"
	result ..= "\n        match self {"
	for _, enum in enum:GetEnumItems() do
		result ..= string.format("\n            Self::%s => %d,", enum.Name, enum.Value)
	end
	result ..= "\n        }"
	result ..= "\n    }"
	result ..= "\n"
	result ..= "\n    pub fn from_num(num: " .. numType .. ") -> Option<Self> {"
	result ..= "\n        match num {"
	for _, enum in enum:GetEnumItems() do
		result ..= string.format("\n            %d => Some(Self::%s),", enum.Value, enum.Name)
	end
	result ..= "\n            _ => None,"
	result ..= "\n        }"
	result ..= "\n    }"
	result ..= "\n}\n"

	result ..= "\nimpl fmt::Display for " .. name .. " {"
	result ..= "\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {"
	result ..= '\n        write!(f, "{}", self.into_name())'
	result ..= "\n    }"
	result ..= "\n}"

	return result
end

contents ..= makeRustEnum(Enum.FontWeight, "u16")
contents ..= "\n"

contents ..= makeRustEnum(Enum.FontStyle, "u8")
contents ..= "\n"

-- Font constant map from enum to font data

-- TODO: Fix formatting issues of output here

local longestNameLen = 0
for _, enum in Enum.Font:GetEnumItems() do
	longestNameLen = math.max(longestNameLen, #enum.Name)
end

contents ..= "\n#[rustfmt::skip]\nconst FONT_ENUM_MAP: &[(&str, Option<FontData>)] = &[\n"
for _, enum in Enum.Font:GetEnumItems() do
	if enum == Enum.Font.Unknown then
		contents ..= '    ("Unknown", None),\n'
	else
		local font = Font.fromEnum(enum)
		contents ..= string.format(
			'    ("%s",%s Some(("%s", FontWeight::%s, FontStyle::%s))),\n',
			enum.Name,
			string.rep(" ", longestNameLen - #enum.Name),
			font.Family,
			font.Weight.Name,
			font.Style.Name
		)
	end
end
contents ..= "];\n"

print(contents)

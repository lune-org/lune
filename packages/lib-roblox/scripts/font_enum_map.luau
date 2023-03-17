-- NOTE: This must be ran in Roblox Studio to get up-to-date font values

local contents = ""

contents ..= "\nconst FONT_ENUM_MAP: &[(&str, Option<(&str, RbxFontWeight, RbxFontStyle)>)] = &[\n"
for _, enum in Enum.Font:GetEnumItems() do
	if enum == Enum.Font.Unknown then
		contents ..= '    ("Unknown", None),\n'
	else
		local font = Font.fromEnum(enum)
		contents ..= string.format(
			'    ("%s", Some(("%s", RbxFontWeight::%s, RbxFontStyle::%s))),\n',
			enum.Name,
			font.Family,
			font.Weight.Name,
			font.Style.Name
		)
	end
end
contents ..= "];\n"

print(contents)

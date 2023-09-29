--!nocheck
--!nolint UnknownGlobal

-- NOTE: This must be ran in Roblox Studio to get up-to-date enum values

local contents = ""

local longestNameLen = 0
for _, enum in Enum.Material:GetEnumItems() do
	longestNameLen = math.max(longestNameLen, #enum.Name)
end

contents ..= "\n#[rustfmt::skip]\nconst MATERIAL_ENUM_MAP: &[(&str, f32, f32, f32, f32, f32)] = &[\n"
for _, enum in Enum.Material:GetEnumItems() do
	local props = PhysicalProperties.new(enum)
	contents ..= string.format(
		'    ("%s",%s %.2f, %.2f, %.2f, %.2f, %.2f),\n',
		enum.Name,
		string.rep(" ", longestNameLen - #enum.Name),
		props.Density,
		props.Friction,
		props.Elasticity,
		props.FrictionWeight,
		props.ElasticityWeight
	)
end
contents ..= "];\n"

print(contents)

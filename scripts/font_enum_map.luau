--!nocheck
--!nolint UnknownGlobal

-- NOTE: This must be ran in Roblox Studio to get up-to-date font values

local contents = ""

contents ..= "\ntype FontData = (&'static str, FontWeight, FontStyle);\n"

local ENUM_IMPLEMENTATION = [[

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum <<ENUM_NAME>> {
	<<ENUM_NAMES>>
}

impl <<ENUM_NAME>> {
	pub(crate) fn as_<<NUMBER_TYPE>>(&self) -> <<NUMBER_TYPE>> {
		match self {
			<<ENUM_TO_NUMBERS>>
		}
	}

	pub(crate) fn from_<<NUMBER_TYPE>>(n: <<NUMBER_TYPE>>) -> Option<Self> {
		match n {
			<<NUMBERS_TO_ENUM>>
			_ => None,
		}
	}
}

impl Default for <<ENUM_NAME>> {
	fn default() -> Self {
		Self::<<DEFAULT_NAME>>
	}
}

impl std::str::FromStr for <<ENUM_NAME>> {
	type Err = &'static str;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			<<STRINGS_TO_ENUM>>
			_ => Err("Unknown <<ENUM_NAME>>"),
		}
	}
}

impl std::fmt::Display for <<ENUM_NAME>> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				<<ENUM_TO_STRINGS>>
			}
		)
	}
}

impl<'lua> FromLua<'lua> for <<ENUM_NAME>> {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let mut message = None;
        if let LuaValue::UserData(ud) = &lua_value {
            let value = ud.borrow::<EnumItem>()?;
            if value.parent.desc.name == "<<ENUM_NAME>>" {
                if let Ok(value) = <<ENUM_NAME>>::from_str(&value.name) {
                    return Ok(value);
                } else {
                    message = Some(format!(
                        "Found unknown Enum.<<ENUM_NAME>> value '{}'",
                        value.name
                    ));
                }
            } else {
                message = Some(format!(
                    "Expected Enum.<<ENUM_NAME>>, got Enum.{}",
                    value.parent.desc.name
                ));
            }
        }
        Err(LuaError::FromLuaConversionError {
            from: lua_value.type_name(),
            to: "Enum.<<ENUM_NAME>>",
            message,
        })
    }
}

impl<'lua> ToLua<'lua> for <<ENUM_NAME>> {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match EnumItem::from_enum_name_and_name("<<ENUM_NAME>>", self.to_string()) {
            Some(enum_item) => Ok(LuaValue::UserData(lua.create_userdata(enum_item)?)),
            None => Err(LuaError::ToLuaConversionError {
                from: "<<ENUM_NAME>>",
                to: "EnumItem",
                message: Some(format!("Found unknown Enum.<<ENUM_NAME>> value '{}'", self)),
            }),
        }
    }
}

]]

-- FontWeight enum and implementation

local function makeRustEnum(enum, default, numType: string)
	local name = tostring(enum)
	name = string.gsub(name, "^Enum.", "")

	local defaultName = tostring(default)
	defaultName = string.gsub(defaultName, "^Enum.", "")
	defaultName = string.gsub(defaultName, "^" .. name .. ".", "")

	local enumNames = ""
	local enumToNumbers = ""
	local numbersToEnum = ""
	local stringsToEnum = ""
	local enumToStrings = ""

	for _, enum in enum:GetEnumItems() do
		enumNames ..= `\n{enum.Name},`
		enumToNumbers ..= `\nSelf::{enum.Name} => {enum.Value},`
		numbersToEnum ..= `\n{enum.Value} => Some(Self::{enum.Name}),`
		stringsToEnum ..= `\n"{enum.Name}" => Ok(Self::{enum.Name}),`
		enumToStrings ..= `\nSelf::{enum.Name} => "{enum.Name}",`
	end

	local mappings: { [string]: string } = {
		["<<ENUM_NAME>>"] = name,
		["<<DEFAULT_NAME>>"] = defaultName,
		["<<NUMBER_TYPE>>"] = numType,
		["<<ENUM_NAMES>>"] = enumNames,
		["<<ENUM_TO_NUMBERS>>"] = enumToNumbers,
		["<<ENUM_TO_STRINGS>>"] = enumToStrings,
		["<<NUMBERS_TO_ENUM>>"] = numbersToEnum,
		["<<STRINGS_TO_ENUM>>"] = stringsToEnum,
	}

	local result = ENUM_IMPLEMENTATION
	for key, replacement in mappings do
		result = string.gsub(result, "(\t*)" .. key, function(tabbing)
			local spacing = string.gsub(tabbing, "\t", "    ")
			local inner = string.gsub(replacement, "\n", "\n" .. spacing)
			inner = string.gsub(inner, "^\n+", "")
			return inner
		end)
	end
	return result
end

contents ..= makeRustEnum(Enum.FontWeight, Enum.FontWeight.Regular, "u16")
contents ..= "\n"

contents ..= makeRustEnum(Enum.FontStyle, Enum.FontStyle.Normal, "u8")
contents ..= "\n"

-- Font constant map from enum to font data

local longestNameLen = 0
local longestFamilyLen = 0
local longestWeightLen = 0
for _, enum in Enum.Font:GetEnumItems() do
	longestNameLen = math.max(longestNameLen, #enum.Name)
	if enum ~= Enum.Font.Unknown then
		local font = Font.fromEnum(enum)
		longestFamilyLen = math.max(longestFamilyLen, #font.Family)
		longestWeightLen = math.max(longestWeightLen, #font.Weight.Name)
	end
end

contents ..= "\n#[rustfmt::skip]\nconst FONT_ENUM_MAP: &[(&str, Option<FontData>)] = &[\n"
for _, enum in Enum.Font:GetEnumItems() do
	if enum == Enum.Font.Unknown then
		contents ..= string.format(
			'    ("Unknown",%s None),\n',
			string.rep(" ", longestNameLen - #enum.Name)
		)
	else
		local font = Font.fromEnum(enum)
		contents ..= string.format(
			'    ("%s",%s Some(("%s",%s FontWeight::%s,%s FontStyle::%s))),\n',
			enum.Name,
			string.rep(" ", longestNameLen - #enum.Name),
			font.Family,
			string.rep(" ", longestFamilyLen - #font.Family),
			font.Weight.Name,
			string.rep(" ", longestWeightLen - #font.Weight.Name),
			font.Style.Name
		)
	end
end
contents ..= "];\n"

print(contents)

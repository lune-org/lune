local TOML_LINES = {
	"[package]",
	'name = "my-cool-toml-package"',
	'version = "0.1.0"',
	"",
	"[values]",
	"epic = true",
	"",
}

local TOML_STRING = table.concat(TOML_LINES, "\n")

local TOML_TABLE = {
	package = {
		name = "my-cool-toml-package",
		version = "0.1.0",
	},
	values = {
		epic = true,
	},
}

return {
	encoded = TOML_STRING,
	decoded = TOML_TABLE,
}

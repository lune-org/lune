--!nocheck
--!nolint UnknownGlobal

local _, err = pcall(function()
	local file = readFile("Cargo.toml")
	if file ~= nil then
		print("Cargo.toml found!")
		print("Contents:")
		print(file)
	else
		print("Cargo.toml not found!")
	end
end)

if err ~= nil then
	print("Error while reading file: " .. err)
end

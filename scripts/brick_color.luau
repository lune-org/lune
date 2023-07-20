local URL =
	"https://gist.githubusercontent.com/Anaminus/49ac255a68e7a7bc3cdd72b602d5071f/raw/f1534dcae312dbfda716b7677f8ac338b565afc3/BrickColor.json"

local json = net.jsonDecode(net.request(URL).body)

local contents = ""

contents ..= "const BRICK_COLOR_DEFAULT: u16 = "
contents ..= tostring(json.Default)
contents ..= ";\n"

contents ..= "\nconst BRICK_COLOR_VALUES: &[(u16, &str, (u8, u8, u8))] = &[\n"
for _, color in json.BrickColors do
	contents ..= string.format(
		'    (%d, "%s", (%d, %d, %d)),\n',
		color.Number,
		color.Name,
		color.Color8[1],
		color.Color8[2],
		color.Color8[3]
	)
end
contents ..= "];\n"

contents ..= "\nconst BRICK_COLOR_PALETTE: &[u16] = &["
contents ..= table.concat(json.Palette, ", ")
contents ..= "];\n"

contents ..= "\nconst BRICK_COLOR_CONSTRUCTORS: &[(&str, u16)] = &["
for key, number in json.Constructors do
	contents ..= string.format('    ("%s", %d),\n', key, number)
end
contents ..= "];\n"

fs.writeFile("packages/lib-roblox/scripts/brick_color.rs", contents)

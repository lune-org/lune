local JSON_STRING = [[{"Hello":"World","Inner":{"Array":[1,2,3]},"Foo":"Bar"}]]

local JSON_STRING_PRETTY = [[{
  "Hello": "World",
  "Inner": {
    "Array": [
      1,
      2,
      3
    ]
  },
  "Foo": "Bar"
}]]

return {
	encoded = JSON_STRING,
	pretty = JSON_STRING_PRETTY,
}

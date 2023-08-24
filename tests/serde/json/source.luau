local JSON_STRING = [[{"Foo":"Bar","Hello":"World","Inner":{"Array":[1,3,2]}}]]

local JSON_STRING_PRETTY = [[{
  "Foo": "Bar",
  "Hello": "World",
  "Inner": {
    "Array": [
      1,
      3,
      2
    ]
  }
}]]

return {
	encoded = JSON_STRING,
	pretty = JSON_STRING_PRETTY,
}

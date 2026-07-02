local JSON_STRING = [[{
  "Foo": "Bar", // Baz ?!?
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
}

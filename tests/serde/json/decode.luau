local net = require("@lune/net")
local serde = require("@lune/serde")

type Response = {
	products: {
		{
			id: number,
			title: string,
			description: string,
			price: number,
			discountPercentage: number,
			rating: number,
			stock: number,
			brand: string,
			category: string,
			thumbnail: string,
			images: { string },
		}
	},
	total: number,
	skip: number,
	limit: number,
}

local response = net.request("https://dummyjson.com/products")

assert(response.ok, "Dummy JSON api returned an error")
assert(#response.body > 0, "Dummy JSON api returned empty body")

local data: Response = serde.decode("json", response.body)

assert(type(data.limit) == "number", "Products limit was not a number")
assert(type(data.products) == "table", "Products was not a table")
assert(#data.products > 0, "Products table was empty")

local productCount = 0
for _, product in data.products do
	productCount += 1
	assert(type(product.id) == "number", "Product id was not a number")
	assert(type(product.title) == "string", "Product title was not a number")
	assert(type(product.description) == "string", "Product description was not a number")
	assert(type(product.images) == "table", "Product images was not a table")
	assert(#product.images > 0, "Product images table was empty")
end

assert(
	data.limit == productCount,
	string.format(
		"Products limit and number of products in array mismatch (expected %d, got %d)",
		data.limit,
		productCount
	)
)

local json5Response = net.request("https://raw.githubusercontent.com/chromium/chromium/feb3c9f670515edf9a88f185301cbd7794ee3e52/third_party/blink/renderer/platform/runtime_enabled_features.json5")

assert(json5Response.ok, "Failed to fetch JSON5 file contents")
assert(#json5Response.body > 0, "Received an empty response body for JSON5 file")

local decodedJson5 = serde.decode("json5", json5Response.body)

assert(type(decodedJson5.parameters) == "table", "Parameters was not a table")
assert(#decodedJson5.data == 556, "Data table wasn't expected size") -- Number is hardcoded, since the commit hash is pinned

